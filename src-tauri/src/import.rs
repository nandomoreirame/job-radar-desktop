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
        let url = row.url.trim();
        if !valid(&row.fingerprint, &row.title, url) {
            report.invalid += 1;
            continue;
        }
        let card = base(
            &row.fingerprint,
            &row.source,
            &row.title,
            &row.company,
            url,
            &date_or(&row.sent_at, today),
        );
        jobs.push(NewJob {
            card,
            url_key: url_key(url),
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
        let url = item.url.trim();
        if !valid(&item.fingerprint, &item.title, url) {
            report.invalid += 1;
            continue;
        }
        let mut card = base(
            &item.fingerprint,
            &item.source,
            &item.title,
            &item.company,
            url,
            &date_or(item.published_at.as_deref().unwrap_or(""), today),
        );
        card.work_model = work(item.work_mode_hint.as_deref());
        card.location = Some(bounded(&item.location, MAX_TEXT)).filter(|v| !v.is_empty());
        card.recruiter_email = Some(item.contact_email.trim().to_owned()).filter(|v| is_email(v));
        jobs.push(NewJob {
            card,
            url_key: url_key(url),
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
    use crate::domain::{Stage, Target, WorkModel};
    use crate::store::Store;
    use rusqlite::{Connection, params};
    use std::path::{Path, PathBuf};

    const FP1: &str = "aaaaaaaaaaaaaaaa";
    const FP2: &str = "bbbbbbbbbbbbbbbb";
    const FP3: &str = "cccccccccccccccc";

    fn seen_db(dir: &Path, rows: &[(&str, &str, &str)]) -> PathBuf {
        let path = dir.join("seen.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE seen (fingerprint TEXT PRIMARY KEY, source TEXT, title TEXT, company TEXT, url TEXT, sent_at TEXT, url_key TEXT, simhash TEXT)",
        )
        .unwrap();
        for (fp, title, url) in rows {
            conn.execute(
                "INSERT INTO seen (fingerprint, source, title, company, url, sent_at) VALUES (?1, 'gupy', ?2, 'Acme', ?3, '2026-10-05T10:00:00.123')",
                params![fp, title, url],
            )
            .unwrap();
        }
        path
    }

    fn candidates(dir: &Path, items: serde_json::Value) -> PathBuf {
        let path = dir.join("candidates.json");
        std::fs::write(&path, items.to_string()).unwrap();
        path
    }

    fn candidate(fp: &str, url: &str) -> serde_json::Value {
        serde_json::json!({
            "fingerprint": fp, "source": "greenhouse", "title": "Data Engineer", "company": "Beta",
            "url": url, "description": "Python e AWS", "location": "Remoto",
            "work_mode_hint": "remote", "published_at": "2026-10-20T08:00:00+00:00",
            "contact_email": "rh@beta.example", "contact_phone": "", "tags": [], "other_locations": [], "external_id": "1"
        })
    }

    fn store() -> crate::sqlite_store::SqliteStore {
        crate::sqlite_store::SqliteStore::open_in_memory().unwrap()
    }

    #[test]
    fn routes_seen_to_inbox_and_candidates_to_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let cand = candidates(
            dir.path(),
            serde_json::json!([candidate(FP2, "https://a.example/2")]),
        );
        let store = store();
        let report = import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        assert_eq!(
            (
                report.inbox,
                report.discarded,
                report.duplicates,
                report.dismissed,
                report.invalid
            ),
            (1, 1, 0, 0, 0)
        );
        let cards = store.list_cards().unwrap();
        assert_eq!(
            (cards[0].id.as_str(), cards[0].stage, cards[0].unread),
            (FP1, Stage::Inbox, true)
        );
        let discarded = store.list_discarded().unwrap();
        assert_eq!(
            (discarded[0].id.as_str(), discarded[0].reason.as_str()),
            (FP2, BELOW_CUT)
        );
    }

    #[test]
    fn maps_a_record_to_a_card() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(
            dir.path(),
            &[(FP1, "Engenheiro de IA", "https://a.example/1")],
        );
        let cand = candidates(
            dir.path(),
            serde_json::json!([candidate(FP2, "https://a.example/2")]),
        );
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        let card = store.list_cards().unwrap().remove(0);
        assert_eq!(card.score, None);
        assert_eq!(card.target, Some(Target::AiLlm));
        assert_eq!(card.work_model, None);
        assert_eq!(card.job_url.as_deref(), Some("https://a.example/1"));
        assert_eq!(card.timeline[0].at, "2026-10-05");
        let job = store.restore_card(FP2).unwrap();
        assert_eq!(job.work_model, Some(WorkModel::Remote));
        assert_eq!(job.recruiter_email.as_deref(), Some("rh@beta.example"));
        assert_eq!(job.location.as_deref(), Some("Remoto"));
        assert_eq!(job.timeline[0].at, "2026-10-20");
    }

    #[test]
    fn importing_twice_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let store = store();
        import_history(&store, &seen, None, "2026-11-02").unwrap();
        store.move_card(FP1, Stage::Triage, None).unwrap();
        let details = crate::domain::CardDetails {
            notes: "minha nota".into(),
            ..Default::default()
        };
        store.update_card(FP1, details).unwrap();
        let before = store.list_cards().unwrap();
        let report = import_history(&store, &seen, None, "2026-11-03").unwrap();
        assert_eq!((report.inbox, report.duplicates), (0, 1));
        assert_eq!(store.list_cards().unwrap(), before);
    }

    #[test]
    fn seen_wins_over_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let cand = candidates(
            dir.path(),
            serde_json::json!([
                candidate(FP1, "https://a.example/other"),
                candidate(FP2, "http://www.a.example/1/")
            ]),
        );
        let store = store();
        let report = import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        assert_eq!(
            (report.inbox, report.discarded, report.duplicates),
            (1, 0, 2)
        );
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn a_deleted_job_does_not_return() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP3, "Outra", "https://a.example/3")]);
        let cand = candidates(
            dir.path(),
            serde_json::json!([candidate(FP2, "https://a.example/2")]),
        );
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        store.delete_card(FP2, "2026-11-02").unwrap();
        let renamed = candidates(
            dir.path(),
            serde_json::json!([candidate(FP1, "https://a.example/2")]),
        );
        let same_id = import_history(&store, &seen, Some(&cand), "2026-11-03").unwrap();
        let same_url = import_history(&store, &seen, Some(&renamed), "2026-11-03").unwrap();
        assert_eq!((same_id.discarded, same_id.dismissed), (0, 1));
        assert_eq!((same_url.discarded, same_url.dismissed), (0, 1));
    }

    #[test]
    fn skips_and_counts_invalid_records() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(
            dir.path(),
            &[
                (FP1, "Boa", "https://a.example/1"),
                (FP2, "Script", "javascript:alert(1)"),
                ("short", "Fingerprint ruim", "https://a.example/3"),
                (FP3, "  ", "https://a.example/4"),
            ],
        );
        let store = store();
        let report = import_history(&store, &seen, None, "2026-11-02").unwrap();
        assert_eq!((report.inbox, report.invalid), (1, 3));
    }

    #[test]
    fn bounds_text_and_drops_an_invalid_email() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[]);
        let mut item = candidate(FP2, "https://a.example/2");
        item["title"] = serde_json::json!("x".repeat(500));
        item["contact_email"] = serde_json::json!("not an email");
        let cand = candidates(dir.path(), serde_json::json!([item]));
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        let job = store.restore_card(FP2).unwrap();
        assert_eq!(job.title.chars().count(), 200);
        assert_eq!(job.recruiter_email, None);
    }

    #[test]
    fn unreadable_candidates_change_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, "{ not json").unwrap();
        let store = store();
        assert!(import_history(&store, &seen, Some(&bad), "2026-11-02").is_err());
        assert!(
            import_history(&store, &dir.path().join("missing.db"), None, "2026-11-02").is_err()
        );
        assert!(store.list_cards().unwrap().is_empty());
    }

    #[test]
    fn a_malformed_candidate_is_skipped_and_counted() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[]);
        let mut broken = candidate(FP3, "https://a.example/3");
        broken["company"] = serde_json::Value::Null;
        let cand = candidates(
            dir.path(),
            serde_json::json!([broken, candidate(FP2, "https://a.example/2")]),
        );
        let report = import_history(&store(), &seen, Some(&cand), "2026-11-02").unwrap();
        assert_eq!((report.discarded, report.invalid), (1, 1));
    }

    #[test]
    fn stores_the_url_without_surrounding_spaces() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "  https://a.example/1  ")]);
        let store = store();
        import_history(&store, &seen, None, "2026-11-02").unwrap();
        assert_eq!(
            store.list_cards().unwrap()[0].job_url.as_deref(),
            Some("https://a.example/1")
        );
    }

    #[test]
    fn an_impossible_date_falls_back_to_today() {
        assert_eq!(date_or("2026-13-99T00:00:00", "2026-11-02"), "2026-11-02");
        assert_eq!(date_or("abcd-ef-ghXX", "2026-11-02"), "2026-11-02");
        assert_eq!(date_or("2028-02-29T10:00:00", "2026-11-02"), "2028-02-29");
    }
}
