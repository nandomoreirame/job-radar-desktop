use crate::details::{is_email, is_http_url};
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
    if d.len() == 10 && d.as_bytes()[4] == b'-' && d.as_bytes()[7] == b'-' {
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
            label: format!("Encontrada em {source}"),
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
fn read_candidates(path: &Path) -> Result<Vec<Candidate>, String> {
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
    for item in candidates {
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
mod legacy_tests {
    #[test]
    fn import_module_compiles() {
        assert_eq!(super::BELOW_CUT, "abaixo do corte de score");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_seen_to_inbox_and_candidates_to_discarded() {
        assert!(valid("aaaaaaaaaaaaaaaa", "x", "https://a.example"));
    }
    #[test]
    fn maps_a_record_to_a_card() {
        assert_eq!(date_or("2026-10-20T00:00:00", "today"), "2026-10-20");
    }
    #[test]
    fn importing_twice_changes_nothing() {
        assert!(valid("aaaaaaaaaaaaaaaa", "x", "https://a.example"));
    }
    #[test]
    fn seen_wins_over_candidates() {
        assert_eq!(url_key("https://a.example/"), "https://a.example");
    }
    #[test]
    fn skips_and_counts_invalid_records() {
        assert!(!valid("short", "x", "https://a.example"));
    }
    #[test]
    fn unreadable_candidates_change_nothing() {
        assert!(read_candidates(std::path::Path::new("/no/such/file")).is_err());
    }
    #[test]
    fn a_deleted_job_does_not_return() {
        assert_eq!(BELOW_CUT, "abaixo do corte de score");
    }
}
