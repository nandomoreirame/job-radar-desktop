//! SQLite backend. Schema changes are append-only migrations.
use crate::domain::{Artifacts, BoardCard, CardDetails, Outcome, Salary, Stage, TimelineEvent};
use crate::store::{Store, StoreError, validate_move};
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

pub const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        StoreError::Storage(error.to_string())
    }
}

pub struct SqliteStore {
    conn: Mutex<Connection>,
}
impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }
    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "foreign_keys", true)?;
        if conn
            .pragma_query_value(None, "journal_mode", |row| row.get::<_, String>(0))
            .unwrap_or_default()
            != "memory"
        {
            let _ = conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| {
                row.get::<_, String>(0)
            });
        }
        conn.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut conn)?;
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
const LIST_CARDS: &str = concat!("SELECT ", card_columns!(), " FROM cards ORDER BY rowid");
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
        target: enum_at(row, 4)?,
        work_model: enum_at(row, 5)?,
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
        to_text(&card.target)?,
        to_text(&card.work_model)?,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    fn user_version(path: &Path) -> i64 {
        Connection::open(path)
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
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
        cards[1].score = 101;
        assert!(matches!(
            store.seed_if_empty(&cards),
            Err(StoreError::Storage(_))
        ));
        assert!(store.list_cards().unwrap().is_empty());
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
        let Err(StoreError::Storage(detail)) = SqliteStore::open(&path) else {
            panic!("a newer schema must be refused")
        };
        assert!(detail.contains("newer"));
        assert_eq!(user_version(&path), 99);
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
    }
}
