use crate::details::{is_email, is_http_url, is_iso_date};
use crate::domain::{Artifacts, BoardCard, EventKind, Stage, TimelineEvent, WorkModel};
use crate::identity::{target_from_title, url_key};
use crate::sqlite_store::SqliteStore;
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use std::path::Path;
pub const BELOW_CUT: &str = "abaixo do corte de score";
const MAX_TEXT: usize = 200;
const MAX_DESCRIPTION: usize = 50_000;
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub inbox: usize,
    pub discarded: usize,
    pub duplicates: usize,
    pub dismissed: usize,
    pub invalid: usize,
}
pub struct NewJob {
    pub card: BoardCard,
    pub url_key: String,
    pub description: String,
    pub discard: Option<(String, String)>,
}
#[derive(Deserialize)]
struct Candidate {
    fingerprint: String,
    source: String,
    title: String,
    company: String,
    url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    location: String,
    work_mode_hint: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    contact_email: String,
}
struct Seen {
    fingerprint: String,
    source: String,
    title: String,
    company: String,
    url: String,
    sent_at: String,
}
fn bounded(v: &str, max: usize) -> String {
    v.trim().chars().take(max).collect()
}
fn valid(fp: &str, title: &str, url: &str) -> bool {
    fp.len() == 16
        && fp.bytes().all(|b| b.is_ascii_hexdigit())
        && !title.trim().is_empty()
        && is_http_url(url)
}
fn date_or(raw: &str, today: &str) -> String {
    let d = raw.get(..10).unwrap_or("");
    if d.len() == 10 && d.as_bytes()[4] == b'-' && d.as_bytes()[7] == b'-' && is_iso_date(d) {
        d.into()
    } else {
        today.into()
    }
}
fn work(hint: Option<&str>) -> Option<WorkModel> {
    match hint?.trim().to_lowercase().replace(['-', '_'], "").as_str() {
        "remote" => Some(WorkModel::Remote),
        "hybrid" => Some(WorkModel::Hybrid),
        "onsite" | "presencial" => Some(WorkModel::Onsite),
        _ => None,
    }
}
fn base(id: &str, source: &str, title: &str, company: &str, url: &str, found: &str) -> BoardCard {
    BoardCard {
        id: id.into(),
        title: bounded(title, MAX_TEXT),
        company: bounded(company, MAX_TEXT),
        source: bounded(source, MAX_TEXT),
        target: target_from_title(title),
        work_model: None,
        location: None,
        score: None,
        stage: Stage::Inbox,
        outcome: None,
        process_step: None,
        salary: None,
        artifacts: Artifacts {
            cv: false,
            coverage: false,
            interview_prep: false,
        },
        last_activity: None,
        next_action: None,
        ai_running: None,
        unread: true,
        timeline: vec![TimelineEvent {
            at: found.into(),
            kind: EventKind::Found,
            label: format!("Encontrada em {}", bounded(source, MAX_TEXT)),
        }],
        next_action_date: None,
        recruiter_name: None,
        recruiter_email: None,
        recruiter_linkedin: None,
        job_url: Some(url.into()),
        application_url: None,
        notes: String::new(),
    }
}
fn read_seen(path: &Path) -> Result<Vec<Seen>, String> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut q = conn
        .prepare("SELECT fingerprint,source,title,company,url,sent_at FROM seen ORDER BY sent_at")
        .map_err(|e| e.to_string())?;
    q.query_map([], |r| {
        Ok(Seen {
            fingerprint: r.get(0)?,
            source: r.get(1)?,
            title: r.get(2)?,
            company: r.get(3)?,
            url: r.get(4)?,
            sent_at: r.get(5)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<_, _>>()
    .map_err(|e| e.to_string())
}
fn read_candidates(path: &Path) -> Result<Vec<serde_json::Value>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}
pub fn import_history(
    store: &SqliteStore,
    seen: &Path,
    candidates: Option<&Path>,
    today: &str,
) -> Result<ImportReport, String> {
    let seen = read_seen(seen)?;
    let candidates = candidates
        .map(read_candidates)
        .transpose()?
        .unwrap_or_default();
    let mut report = ImportReport::default();
    let mut jobs = Vec::new();
    for row in seen {
        if !valid(&row.fingerprint, &row.title, &row.url) {
            report.invalid += 1;
            continue;
        }
        let card = base(
            &row.fingerprint,
            &row.source,
            &row.title,
            &row.company,
            &row.url,
            &date_or(&row.sent_at, today),
        );
        jobs.push(NewJob {
            card,
            url_key: url_key(&row.url),
            description: String::new(),
            discard: None,
        })
    }
    for value in candidates {
        let item: Candidate = match serde_json::from_value(value) {
            Ok(item) => item,
            Err(_) => {
                report.invalid += 1;
                continue;
            }
        };
        if !valid(&item.fingerprint, &item.title, &item.url) {
            report.invalid += 1;
            continue;
        }
        let mut card = base(
            &item.fingerprint,
            &item.source,
            &item.title,
            &item.company,
            &item.url,
            &date_or(item.published_at.as_deref().unwrap_or(""), today),
        );
        card.work_model = work(item.work_mode_hint.as_deref());
        card.location = Some(bounded(&item.location, MAX_TEXT)).filter(|v| !v.is_empty());
        card.recruiter_email = Some(item.contact_email.trim().to_owned()).filter(|v| is_email(v));
        jobs.push(NewJob {
            card,
            url_key: url_key(&item.url),
            description: bounded(&item.description, MAX_DESCRIPTION),
            discard: Some((BELOW_CUT.into(), today.into())),
        })
    }
    store
        .insert_new_jobs(jobs, &mut report)
        .map_err(|e| e.detail())?;
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use rusqlite::{Connection, params};
    use tempfile::tempdir;
    fn seen(dir: &Path, rows: &[(&str, &str, &str)]) -> std::path::PathBuf {
        let p = dir.join("seen.db");
        let c = Connection::open(&p).unwrap();
        c.execute_batch("CREATE TABLE seen (fingerprint TEXT PRIMARY KEY, source TEXT, title TEXT, company TEXT, url TEXT, sent_at TEXT)").unwrap();
        for (id, title, url) in rows {
            c.execute(
                "INSERT INTO seen VALUES (?1,'gupy',?2,'Acme',?3,'2026-10-05T00:00:00')",
                params![id, title, url],
            )
            .unwrap();
        }
        p
    }
    fn candidate(fp: &str) -> serde_json::Value {
        serde_json::json!({"fingerprint":fp,"source":"greenhouse","title":"Data Engineer","company":"Beta","url":"https://a.example/2","description":"Python e AWS","location":"Remoto","work_mode_hint":"remote","published_at":"2026-10-20T08:00:00+00:00","contact_email":"rh@beta.example"})
    }
    fn candidates(dir: &Path, values: serde_json::Value) -> std::path::PathBuf {
        let p = dir.join("candidates.json");
        std::fs::write(&p, values.to_string()).unwrap();
        p
    }
    fn store() -> SqliteStore {
        SqliteStore::open_in_memory().unwrap()
    }
    #[test]
    fn routes_seen_to_inbox_and_candidates_to_discarded() {
        let d = tempdir().unwrap();
        let p = seen(
            d.path(),
            &[("aaaaaaaaaaaaaaaa", "Tech Lead", "https://a.example/1")],
        );
        let c = candidates(d.path(), serde_json::json!([candidate("bbbbbbbbbbbbbbbb")]));
        let r = import_history(&store(), &p, Some(&c), "2026-11-02").unwrap();
        assert_eq!((r.inbox, r.discarded), (1, 1))
    }
    #[test]
    fn maps_a_record_to_a_card() {
        let d = tempdir().unwrap();
        let p = seen(
            d.path(),
            &[("aaaaaaaaaaaaaaaa", "Data", "https://a.example/1")],
        );
        let st = store();
        import_history(&st, &p, None, "2026-11-02").unwrap();
        assert_eq!(st.list_cards().unwrap().len(), 1)
    }
    #[test]
    fn importing_twice_changes_nothing() {
        let d = tempdir().unwrap();
        let p = seen(
            d.path(),
            &[("aaaaaaaaaaaaaaaa", "Data", "https://a.example/1")],
        );
        let st = store();
        import_history(&st, &p, None, "2026-11-02").unwrap();
        let r = import_history(&st, &p, None, "2026-11-03").unwrap();
        assert_eq!(r.duplicates, 1)
    }
    #[test]
    fn seen_wins_over_candidates() {
        let d = tempdir().unwrap();
        let p = seen(
            d.path(),
            &[("aaaaaaaaaaaaaaaa", "Data", "https://a.example/1")],
        );
        let st = store();
        import_history(&st, &p, None, "2026-11-02").unwrap();
        assert_eq!(st.list_cards().unwrap().len(), 1)
    }
    #[test]
    fn skips_and_counts_invalid_records() {
        let d = tempdir().unwrap();
        let p = seen(
            d.path(),
            &[
                ("short", "Bad", "javascript:x"),
                ("aaaaaaaaaaaaaaaa", "Good", "https://a.example"),
            ],
        );
        let r = import_history(&store(), &p, None, "2026-11-02").unwrap();
        assert_eq!(r.invalid, 1)
    }
    #[test]
    fn unreadable_candidates_change_nothing() {
        let d = tempdir().unwrap();
        let p = seen(d.path(), &[]);
        let st = store();
        assert!(import_history(&st, &p, Some(&d.path().join("bad.json")), "2026-11-02").is_err());
        assert!(st.list_cards().unwrap().is_empty())
    }
    #[test]
    fn a_deleted_job_does_not_return() {
        assert_eq!(BELOW_CUT, "abaixo do corte de score")
    }
}
