//! SQLite backend. Schema changes are append-only migrations.
use crate::domain::{
    Artifacts, BoardCard, CardDetails, CrawlRun, DiscardedJob, Outcome, Salary, Stage, Target,
    TimelineEvent, WorkModel,
};
use crate::import::{ImportReport, NewJob};
use crate::store::{Store, StoreError, validate_move};
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

pub const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_init.sql"),
    include_str!("../migrations/002_real_jobs.sql"),
    include_str!("../migrations/003_crawl_runs.sql"),
];

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        StoreError::Storage(error.to_string())
    }
}

pub struct SqliteStore {
    conn: Mutex<Connection>,
}
impl SqliteStore {
    pub fn insert_new_jobs(
        &self,
        jobs: Vec<NewJob>,
        report: &mut ImportReport,
    ) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let (mut ids, mut urls) = read_keys(&tx, CARD_KEYS)?;
        let (dismissed, dismissed_urls) = read_keys(&tx, DISMISSED_KEYS)?;
        for job in jobs {
            let key = (!job.url_key.is_empty()).then_some(job.url_key.as_str());
            if dismissed.contains(&job.card.id) || key.is_some_and(|v| dismissed_urls.contains(v)) {
                report.dismissed += 1;
                continue;
            }
            if ids.contains(&job.card.id) || key.is_some_and(|v| urls.contains(v)) {
                report.duplicates += 1;
                continue;
            }
            insert_card(&tx, &job.card)?;
            let (reason, at) = job.discard.unzip();
            tx.execute("UPDATE cards SET url_key=?2,description=?3,discard_reason=?4,discarded_at=?5 WHERE id=?1",params![job.card.id,key,job.description,reason,at])?;
            ids.insert(job.card.id);
            if let Some(v) = key {
                urls.insert(v.to_owned());
            }
            if reason.is_some() {
                report.discarded += 1
            } else {
                report.inbox += 1
            }
        }
        tx.commit()?;
        Ok(())
    }
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }
    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        // Refuse newer schemas before any write pragma can alter the file.
        migrate(&mut conn)?;
        conn.pragma_update(None, "foreign_keys", true)?;
        let journal_mode: String =
            conn.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
        if journal_mode != "memory" {
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| {
                row.get::<_, String>(0)
            })?;
        }
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }
}
fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
    let current = usize::try_from(current).unwrap_or(usize::MAX);
    if current > MIGRATIONS.len() {
        return Err(StoreError::Storage(format!(
            "database schema v{current} is newer than this app supports"
        )));
    }
    if current == MIGRATIONS.len() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for sql in &MIGRATIONS[current..] {
        tx.execute_batch(sql)?;
    }
    tx.pragma_update(None, "user_version", MIGRATIONS.len() as i64)?;
    tx.commit()?;
    Ok(())
}

macro_rules! card_columns { () => { "id, title, company, source, target, work_model, location, score, stage, outcome, process_step, salary_pj, salary_clt, has_cv, has_coverage, has_interview_prep, last_activity, next_action, next_action_date, ai_running, unread, recruiter_name, recruiter_email, recruiter_linkedin, job_url, application_url, notes" }; }
const LIST_CARDS: &str = concat!(
    "SELECT ",
    card_columns!(),
    " FROM cards WHERE discard_reason IS NULL ORDER BY rowid"
);
const GET_CARD: &str = concat!("SELECT ", card_columns!(), " FROM cards WHERE id = ?1");
const INSERT_CARD: &str = concat!(
    "INSERT INTO cards (",
    card_columns!(),
    ") VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27)"
);
const LIST_EVENTS: &str =
    "SELECT card_id, at, kind, label FROM timeline_events ORDER BY card_id, at, id";
const CARD_EVENTS: &str =
    "SELECT card_id, at, kind, label FROM timeline_events WHERE card_id = ?1 ORDER BY at, id";
const INSERT_EVENT: &str =
    "INSERT INTO timeline_events (card_id, at, kind, label) VALUES (?1, ?2, ?3, ?4)";
fn to_text<T: Serialize>(value: &T) -> Result<String, StoreError> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(text)) => Ok(text),
        _ => Err(StoreError::Storage("enum did not serialize to text".into())),
    }
}
fn opt_enum_at<T: DeserializeOwned>(row: &Row<'_>, idx: usize) -> rusqlite::Result<Option<T>> {
    match row.get::<_, Option<String>>(idx)? {
        Some(_) => enum_at(row, idx).map(Some),
        None => Ok(None),
    }
}
fn enum_at<T: DeserializeOwned>(row: &Row<'_>, idx: usize) -> rusqlite::Result<T> {
    let text: String = row.get(idx)?;
    serde_json::from_value(serde_json::Value::String(text))
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, Box::new(e)))
}
fn card_from_row(row: &Row<'_>) -> rusqlite::Result<BoardCard> {
    let pj: Option<u32> = row.get(11)?;
    let clt: Option<u32> = row.get(12)?;
    Ok(BoardCard {
        id: row.get(0)?,
        title: row.get(1)?,
        company: row.get(2)?,
        source: row.get(3)?,
        target: opt_enum_at(row, 4)?,
        work_model: opt_enum_at(row, 5)?,
        location: row.get(6)?,
        score: row.get(7)?,
        stage: enum_at(row, 8)?,
        outcome: match row.get::<_, Option<String>>(9)? {
            Some(_) => Some(enum_at(row, 9)?),
            None => None,
        },
        process_step: row.get(10)?,
        salary: (pj.is_some() || clt.is_some()).then_some(Salary { pj, clt }),
        artifacts: Artifacts {
            cv: row.get(13)?,
            coverage: row.get(14)?,
            interview_prep: row.get(15)?,
        },
        last_activity: row.get(16)?,
        next_action: row.get(17)?,
        next_action_date: row.get(18)?,
        ai_running: row.get(19)?,
        unread: row.get(20)?,
        recruiter_name: row.get(21)?,
        recruiter_email: row.get(22)?,
        recruiter_linkedin: row.get(23)?,
        job_url: row.get(24)?,
        application_url: row.get(25)?,
        notes: row.get(26)?,
        timeline: Vec::new(),
    })
}
fn event_from_row(row: &Row<'_>) -> rusqlite::Result<(String, TimelineEvent)> {
    Ok((
        row.get(0)?,
        TimelineEvent {
            at: row.get(1)?,
            kind: enum_at(row, 2)?,
            label: row.get(3)?,
        },
    ))
}
fn insert_card(conn: &Connection, card: &BoardCard) -> Result<(), StoreError> {
    let salary = card.salary.as_ref();
    conn.prepare_cached(INSERT_CARD)?.execute(params![
        card.id,
        card.title,
        card.company,
        card.source,
        card.target.map(|value| to_text(&value)).transpose()?,
        card.work_model.map(|value| to_text(&value)).transpose()?,
        card.location,
        card.score,
        to_text(&card.stage)?,
        card.outcome.map(|o| to_text(&o)).transpose()?,
        card.process_step,
        salary.and_then(|s| s.pj),
        salary.and_then(|s| s.clt),
        card.artifacts.cv,
        card.artifacts.coverage,
        card.artifacts.interview_prep,
        card.last_activity,
        card.next_action,
        card.next_action_date,
        card.ai_running,
        card.unread,
        card.recruiter_name,
        card.recruiter_email,
        card.recruiter_linkedin,
        card.job_url,
        card.application_url,
        card.notes
    ])?;
    let mut events = conn.prepare_cached(INSERT_EVENT)?;
    for event in &card.timeline {
        events.execute(params![
            card.id,
            event.at,
            to_text(&event.kind)?,
            event.label
        ])?;
    }
    Ok(())
}
fn load_card(conn: &Connection, id: &str) -> Result<BoardCard, StoreError> {
    let mut card = conn
        .prepare_cached(GET_CARD)?
        .query_row(params![id], card_from_row)
        .optional()?
        .ok_or_else(|| StoreError::NotFound(id.to_owned()))?;
    card.timeline = conn
        .prepare_cached(CARD_EVENTS)?
        .query_map(params![id], event_from_row)?
        .map(|row| row.map(|(_, event)| event))
        .collect::<Result<_, _>>()?;
    Ok(card)
}
pub struct ScoreUpdate {
    pub score: u8,
    pub target: Option<Target>,
    pub work_model: Option<WorkModel>,
    pub location: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnscoredCard {
    pub id: String,
    pub source: String,
    pub title: String,
    pub company: String,
    pub job_url: Option<String>,
    pub description: String,
    pub location: Option<String>,
    pub stage: Stage,
    pub work_model: Option<WorkModel>,
}

/// Everything already seen, by fingerprint and by URL key, deleted jobs included.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownKeys {
    pub ids: HashSet<String>,
    pub urls: HashSet<String>,
}

const CARD_KEYS: &str = "SELECT id,url_key FROM cards";
const DISMISSED_KEYS: &str = "SELECT fingerprint,url_key FROM dismissed_jobs";
/// `COALESCE` keeps stored values: a null update never erases data.
const APPLY_SCORE: &str = "UPDATE cards SET score = ?2, target = COALESCE(target, ?3), work_model = COALESCE(work_model, ?4), \
    location = COALESCE(location, ?5), description = CASE WHEN ?6 = '' THEN description ELSE ?6 END WHERE id = ?1";
const UNSCORED_CARDS: &str = "SELECT id,source,title,company,job_url,description,location,stage,work_model FROM cards \
    WHERE score IS NULL AND discard_reason IS NULL ORDER BY rowid";
const UNSCORED_DISCARDED: &str = "SELECT id,source,title,company,job_url,description,location,stage,work_model FROM cards \
    WHERE score IS NULL AND discard_reason IS NOT NULL AND description != '' ORDER BY rowid";

fn read_keys(
    conn: &Connection,
    sql: &str,
) -> Result<(HashSet<String>, HashSet<String>), StoreError> {
    let mut ids = HashSet::new();
    let mut urls = HashSet::new();
    for row in conn.prepare(sql)?.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
    })? {
        let (id, url) = row?;
        ids.insert(id);
        urls.extend(url);
    }
    Ok((ids, urls))
}

fn unscored_from_row(row: &Row<'_>) -> rusqlite::Result<UnscoredCard> {
    Ok(UnscoredCard {
        id: row.get(0)?,
        source: row.get(1)?,
        title: row.get(2)?,
        company: row.get(3)?,
        job_url: row.get(4)?,
        description: row.get(5)?,
        location: row.get(6)?,
        stage: enum_at(row, 7)?,
        work_model: opt_enum_at(row, 8)?,
    })
}

#[cfg(test)]
impl SqliteStore {
    pub fn seed_if_empty(&self, cards: &[BoardCard]) -> Result<bool, StoreError> {
        let mut conn = self.lock()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(false);
        }
        let tx = conn.transaction()?;
        for card in cards {
            insert_card(&tx, card)?;
        }
        tx.commit()?;
        Ok(true)
    }
}

impl SqliteStore {
    pub fn apply_score(&self, id: &str, update: &ScoreUpdate) -> Result<(), StoreError> {
        let changed = self.lock()?.execute(
            APPLY_SCORE,
            params![
                id,
                update.score,
                update.target.map(|value| to_text(&value)).transpose()?,
                update.work_model.map(|value| to_text(&value)).transpose()?,
                update.location,
                update.description,
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        Ok(())
    }

    fn unscored(&self, sql: &str) -> Result<Vec<UnscoredCard>, StoreError> {
        Ok(self
            .lock()?
            .prepare(sql)?
            .query_map([], unscored_from_row)?
            .collect::<Result<_, _>>()?)
    }

    pub fn unscored_cards(&self) -> Result<Vec<UnscoredCard>, StoreError> {
        self.unscored(UNSCORED_CARDS)
    }

    pub fn unscored_discarded(&self) -> Result<Vec<UnscoredCard>, StoreError> {
        self.unscored(UNSCORED_DISCARDED)
    }

    pub fn start_run(&self, at: &str) -> Result<i64, StoreError> {
        let conn = self.lock()?;
        conn.execute("INSERT INTO crawl_runs(started_at) VALUES(?1)", params![at])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn finish_run(&self, id: i64, at: &str, stats: &str) -> Result<(), StoreError> {
        let changed = self.lock()?.execute(
            "UPDATE crawl_runs SET finished_at=?2,stats=?3 WHERE id=?1",
            params![id, at, stats],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn last_run(&self) -> Result<Option<CrawlRun>, StoreError> {
        Ok(self
            .lock()?
            .query_row(
                "SELECT started_at,finished_at,stats FROM crawl_runs ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok(CrawlRun {
                        started_at: row.get(0)?,
                        finished_at: row.get(1)?,
                        stats: row.get(2)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn known_keys(&self) -> Result<KnownKeys, StoreError> {
        let conn = self.lock()?;
        let (mut ids, mut urls) = read_keys(&conn, CARD_KEYS)?;
        let (dismissed, dismissed_urls) = read_keys(&conn, DISMISSED_KEYS)?;
        ids.extend(dismissed);
        urls.extend(dismissed_urls);
        Ok(KnownKeys { ids, urls })
    }
}
impl Store for SqliteStore {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError> {
        let conn = self.lock()?;
        let mut cards: Vec<BoardCard> = conn
            .prepare_cached(LIST_CARDS)?
            .query_map([], card_from_row)?
            .collect::<Result<_, _>>()?;
        let mut events: HashMap<String, Vec<TimelineEvent>> = HashMap::new();
        for row in conn
            .prepare_cached(LIST_EVENTS)?
            .query_map([], event_from_row)?
        {
            let (id, event) = row?;
            events.entry(id).or_default().push(event)
        }
        for card in &mut cards {
            card.timeline = events.remove(&card.id).unwrap_or_default()
        }
        Ok(cards)
    }
    fn move_card(
        &self,
        id: &str,
        stage: Stage,
        outcome: Option<Outcome>,
    ) -> Result<BoardCard, StoreError> {
        validate_move(stage, outcome)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "UPDATE cards SET stage=?2,outcome=?3 WHERE id=?1",
            params![
                id,
                to_text(&stage)?,
                outcome.map(|o| to_text(&o)).transpose()?
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }
    fn update_card(&self, id: &str, details: CardDetails) -> Result<BoardCard, StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let salary = details.salary.as_ref();
        let changed=tx.execute("UPDATE cards SET salary_pj=?2,salary_clt=?3,process_step=?4,next_action=?5,next_action_date=?6,recruiter_name=?7,recruiter_email=?8,recruiter_linkedin=?9,job_url=?10,application_url=?11,notes=?12 WHERE id=?1",params![id,salary.and_then(|s|s.pj),salary.and_then(|s|s.clt),details.process_step,details.next_action,details.next_action_date,details.recruiter_name,details.recruiter_email,details.recruiter_linkedin,details.job_url,details.application_url,details.notes])?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }

    fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError> {
        let conn = self.lock()?;
        Ok(conn.prepare("SELECT id,title,company,source,location,score,discard_reason,discarded_at,job_url,description FROM cards WHERE discard_reason IS NOT NULL ORDER BY discarded_at DESC,rowid")?.query_map([], |row| Ok(DiscardedJob{id:row.get(0)?,title:row.get(1)?,company:row.get(2)?,source:row.get(3)?,location:row.get(4)?,score:row.get(5)?,reason:row.get(6)?,discarded_at:row.get(7)?,job_url:row.get(8)?,description:row.get(9)?}))?.collect::<Result<_,_>>()?)
    }
    fn discard_card(&self, id: &str, reason: &str, at: &str) -> Result<(), StoreError> {
        if self.lock()?.execute("UPDATE cards SET discard_reason=?2,discarded_at=?3 WHERE id=?1 AND discard_reason IS NULL",params![id,reason,at])?==0{return Err(StoreError::NotFound(id.into()))}
        Ok(())
    }
    fn restore_card(&self, id: &str) -> Result<BoardCard, StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        if tx.execute("UPDATE cards SET discard_reason=NULL,discarded_at=NULL WHERE id=?1 AND discard_reason IS NOT NULL",params![id])?==0{return Err(StoreError::NotFound(id.into()))}
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }
    fn delete_card(&self, id: &str, at: &str) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let key: Option<Option<String>> = tx
            .query_row(
                "SELECT url_key FROM cards WHERE id=?1 AND discard_reason IS NOT NULL",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(key) = key else {
            return Err(StoreError::NotFound(id.into()));
        };
        tx.execute("INSERT OR REPLACE INTO dismissed_jobs(fingerprint,url_key,dismissed_at) VALUES(?1,?2,?3)",params![id,key,at])?;
        tx.execute("DELETE FROM cards WHERE id=?1", params![id])?;
        tx.commit()?;
        Ok(())
    }
    fn card_url(&self, id: &str) -> Result<Option<String>, StoreError> {
        self.lock()?
            .query_row(
                "SELECT job_url FROM cards WHERE id=?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| StoreError::NotFound(id.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    #[test]
    fn the_last_run_survives_a_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            assert_eq!(store.last_run().unwrap(), None);
            let id = store.start_run("2026-11-02T10:00:00-03:00").unwrap();
            assert!(store.last_run().unwrap().unwrap().finished_at.is_none());
            store
                .finish_run(id, "2026-11-02T10:01:00-03:00", r#"{"found":3}"#)
                .unwrap();
        }
        let run = SqliteStore::open(&path)
            .unwrap()
            .last_run()
            .unwrap()
            .unwrap();
        assert_eq!(
            run.finished_at.as_deref(),
            Some("2026-11-02T10:01:00-03:00")
        );
        assert_eq!(run.stats, r#"{"found":3}"#);
    }

    #[test]
    fn applying_a_score_keeps_user_fields() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.score = None;
        card.notes = "minha nota".into();
        card.work_model = None;
        card.location = None;
        store.seed_if_empty(std::slice::from_ref(&card)).unwrap();
        store.move_card(&card.id, Stage::Triage, None).unwrap();
        let scoring = ScoreUpdate {
            score: 85,
            target: None,
            work_model: Some(crate::domain::WorkModel::Hybrid),
            location: Some("Recife".into()),
            description: "texto".into(),
        };
        store.apply_score(&card.id, &scoring).unwrap();
        let stored = store.list_cards().unwrap().remove(0);
        assert_eq!(
            (stored.score, stored.stage, stored.notes.as_str()),
            (Some(85), Stage::Triage, "minha nota")
        );
        assert_eq!(stored.work_model, Some(crate::domain::WorkModel::Hybrid));
        assert_eq!(stored.location.as_deref(), Some("Recife"));
        // Target already known stays; a null update never erases data.
        assert_eq!(stored.target, card.target);
        // A second update fills nothing already stored, and an empty text keeps the old one.
        let again = ScoreUpdate {
            score: 40,
            target: None,
            work_model: Some(crate::domain::WorkModel::Onsite),
            location: Some("Natal".into()),
            description: String::new(),
        };
        store.apply_score(&card.id, &again).unwrap();
        let stored = store.list_cards().unwrap().remove(0);
        assert_eq!(stored.score, Some(40));
        assert_eq!(stored.work_model, Some(crate::domain::WorkModel::Hybrid));
        assert_eq!(stored.location.as_deref(), Some("Recife"));
        let pending = store.unscored_cards().unwrap();
        assert!(pending.is_empty());
        store.discard_card(&card.id, "x", "2026-11-02").unwrap();
        assert_eq!(store.list_discarded().unwrap()[0].description, "texto");
        store.restore_card(&card.id).unwrap();
        assert_eq!(
            store.apply_score("missing", &again),
            Err(StoreError::NotFound("missing".into()))
        );
    }

    #[test]
    fn lists_only_cards_needing_a_score() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut cards = crate::mock_data::sample_cards();
        cards[0].score = None;
        store.seed_if_empty(&cards).unwrap();
        store.discard_card(&cards[1].id, "x", "2026-11-02").unwrap();
        let pending = store.unscored_cards().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, cards[0].id);
    }

    fn new_job(id: &str, url: &str, description: &str, discard: bool) -> NewJob {
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.id = id.into();
        card.score = None;
        NewJob {
            card,
            url_key: url.into(),
            description: description.into(),
            discard: discard.then(|| ("x".to_owned(), "2026-11-02".to_owned())),
        }
    }

    #[test]
    fn lists_discarded_jobs_with_text_to_score() {
        let store = SqliteStore::open_in_memory().unwrap();
        let jobs = vec![
            new_job("aaaaaaaaaaaaaaaa", "https://a.example/1", "texto", true),
            new_job("bbbbbbbbbbbbbbbb", "https://a.example/2", "", true),
            new_job("cccccccccccccccc", "https://a.example/3", "texto", false),
        ];
        store
            .insert_new_jobs(jobs, &mut crate::import::ImportReport::default())
            .unwrap();
        let pending = store.unscored_discarded().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "aaaaaaaaaaaaaaaa");
        assert_eq!(pending[0].description, "texto");
    }

    #[test]
    fn known_keys_cover_cards_and_deleted_jobs() {
        let store = SqliteStore::open_in_memory().unwrap();
        let jobs = vec![
            new_job("aaaaaaaaaaaaaaaa", "https://a.example/1", "", false),
            new_job("bbbbbbbbbbbbbbbb", "https://a.example/2", "", true),
        ];
        store
            .insert_new_jobs(jobs, &mut crate::import::ImportReport::default())
            .unwrap();
        store.delete_card("bbbbbbbbbbbbbbbb", "2026-11-02").unwrap();
        let known = store.known_keys().unwrap();
        assert!(known.ids.contains("aaaaaaaaaaaaaaaa") && known.ids.contains("bbbbbbbbbbbbbbbb"));
        assert!(
            known.urls.contains("https://a.example/1")
                && known.urls.contains("https://a.example/2")
        );
    }

    fn user_version(path: &Path) -> i64 {
        Connection::open(path)
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }
    #[test]
    fn migration_2_keeps_existing_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATIONS[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            insert_card(&conn, &crate::mock_data::sample_cards()[6]).unwrap();
        }
        let expected = crate::mock_data::sample_cards()[6].clone();
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.list_cards().unwrap(), vec![expected]);
        let keys: i64 = store
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE description = '' AND discard_reason IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(keys, 1);
    }
    #[test]
    fn deleting_keeps_a_tombstone_and_removes_the_timeline() {
        let store = SqliteStore::open_in_memory().unwrap();
        store
            .seed_if_empty(&crate::mock_data::sample_cards())
            .unwrap();
        store.discard_card("c01", "x", "2026-11-02").unwrap();
        store.delete_card("c01", "2026-11-02").unwrap();
        let conn = store.lock().unwrap();
        let tombstones: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dismissed_jobs WHERE fingerprint = 'c01'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let events: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM timeline_events WHERE card_id = 'c01'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!((tombstones, events), (1, 0));
    }

    #[test]
    fn a_card_without_score_round_trips() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.score = None;
        card.target = None;
        card.work_model = None;
        store.seed_if_empty(std::slice::from_ref(&card)).unwrap();
        assert_eq!(store.list_cards().unwrap(), vec![card]);
    }

    #[test]
    fn migrates_once_to_the_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
    }
    #[test]
    fn sqlite_store_honors_the_store_contract() {
        crate::store_contract::check_all(|| {
            let store = SqliteStore::open_in_memory().unwrap();
            store
                .seed_if_empty(&crate::mock_data::sample_cards())
                .unwrap();
            Box::new(store)
        });
    }
    #[test]
    fn seeds_only_an_empty_database() {
        let store = SqliteStore::open_in_memory().unwrap();
        assert!(
            store
                .seed_if_empty(&crate::mock_data::sample_cards())
                .unwrap()
        );
        store
            .move_card("c01", crate::domain::Stage::Triage, None)
            .unwrap();
        assert!(
            !store
                .seed_if_empty(&crate::mock_data::sample_cards())
                .unwrap()
        );
        assert_eq!(
            store.list_cards().unwrap().len(),
            crate::mock_data::sample_cards().len()
        );
    }
    #[test]
    fn a_failed_write_leaves_the_data_intact() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut cards = crate::mock_data::sample_cards();
        cards[1].score = None;
        store.seed_if_empty(&cards).unwrap();
        assert_eq!(store.list_cards().unwrap().len(), cards.len());
    }
    #[test]
    fn unknown_stored_values_are_a_storage_error() {
        let store = SqliteStore::open_in_memory().unwrap();
        store
            .seed_if_empty(&crate::mock_data::sample_cards())
            .unwrap();
        {
            let conn = store.lock().unwrap();
            conn.pragma_update(None, "ignore_check_constraints", true)
                .unwrap();
            conn.execute("UPDATE cards SET target = 'astronaut' WHERE id = 'c01'", [])
                .unwrap();
        }
        assert!(matches!(store.list_cards(), Err(StoreError::Storage(_))));
    }

    #[test]
    fn refuses_a_newer_schema_without_touching_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        let Err(StoreError::Storage(detail)) = SqliteStore::open(&path) else {
            panic!("a newer schema must be refused")
        };
        assert!(detail.contains("newer"));
        assert_eq!(user_version(&path), 99);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let tables: i64 = Connection::open(&path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0);
    }
    #[test]
    fn moves_and_edits_survive_a_reopen() {
        use crate::domain::{CardDetails, Outcome, Stage};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            store
                .seed_if_empty(&crate::mock_data::sample_cards())
                .unwrap();
            store
                .move_card("c02", Stage::Closed, Some(Outcome::Rejected))
                .unwrap();
            store
                .update_card(
                    "c03",
                    CardDetails {
                        notes: "Retornar em dezembro".into(),
                        ..CardDetails::default()
                    },
                )
                .unwrap();
        }
        let cards = SqliteStore::open(&path).unwrap().list_cards().unwrap();
        let c02 = cards.iter().find(|c| c.id == "c02").unwrap();
        assert_eq!(
            (c02.stage, c02.outcome),
            (Stage::Closed, Some(Outcome::Rejected))
        );
        assert_eq!(
            cards.iter().find(|c| c.id == "c03").unwrap().notes,
            "Retornar em dezembro"
        );
        assert_eq!(cards.len(), crate::mock_data::sample_cards().len());
    }
    #[cfg(unix)]
    #[test]
    fn database_files_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data").join("job-radar.db");
        crate::db_path::prepare(&path).unwrap();
        let store = SqliteStore::open(&path).unwrap();
        store
            .seed_if_empty(&crate::mock_data::sample_cards())
            .unwrap();
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(&path.with_extension("db-wal")), 0o600);
    }
}
