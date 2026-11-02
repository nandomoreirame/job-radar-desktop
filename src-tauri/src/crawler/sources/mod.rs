//! One connector per file: pure parsing tested on recorded pages, plus fetching.

pub mod gupy;
pub mod solides;

use chrono::{FixedOffset, NaiveDate, NaiveDateTime};
use serde_json::Value;

use crate::clock::Now;
use crate::crawler::http::HttpError;
use crate::crawler::job::Job;

/// What a job's page says. Only positive evidence closes a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailOutcome {
    Found(JobDetail),
    Closed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobDetail {
    pub description: String,
    pub location: String,
    pub work_mode_hint: Option<String>,
    pub published_at: Option<Now>,
}

/// `keep` decides which listed jobs deserve a detail request, as in `collect.py`.
pub trait Source {
    fn name(&self) -> &'static str;
    /// Owner of a stored `source` value such as `gupy` or `github:backend-br`.
    fn owns(&self, source: &str) -> bool;
    fn collect(
        &self,
        keep: &dyn Fn(&Job) -> bool,
        progress: &dyn Fn(&str),
    ) -> Result<Vec<Job>, String>;
    fn fetch_detail(&self, url: &str) -> DetailOutcome;
}

/// Only 404 and 410 say the ad is gone; a block or a timeout says nothing about it.
fn error_outcome(error: HttpError) -> DetailOutcome {
    match error {
        HttpError::Status(404 | 410) => DetailOutcome::Closed,
        other => DetailOutcome::Failed(format!("{other:?}")),
    }
}

fn text<'a>(raw: &'a Value, key: &str) -> &'a str {
    raw[key].as_str().unwrap_or("")
}

/// Python's `str(raw.get("id", ""))`: sources send numbers or strings.
fn id_text(raw: &Value) -> String {
    match &raw["id"] {
        Value::Null => String::new(),
        Value::String(id) => id.clone(),
        other => other.to_string(),
    }
}

/// Python's `datetime.fromisoformat`. A date without an offset is local time;
/// both portals are Brazilian, so that means -03:00.
fn parse_iso(raw: &str) -> Option<Now> {
    let raw = raw.trim();
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(parsed);
    }
    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.f")
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(raw, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)
        })?;
    naive
        .and_local_timezone(FixedOffset::west_opt(3 * 3600)?)
        .single()
}

#[cfg(test)]
pub(crate) mod tests_support {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use serde_json::Value;

    use crate::crawler::http::{RawResponse, Transport};
    use crate::crawler::job::Job;

    pub fn vectors() -> Value {
        serde_json::from_str(include_str!("../../../tests/fixtures/parity/sources.json")).unwrap()
    }

    pub fn assert_jobs_match(jobs: &[Job], key: &str) {
        let v = vectors();
        let expected = v[key].as_array().unwrap();
        assert_eq!(jobs.len(), expected.len(), "{key}");
        for (job, row) in jobs.iter().zip(expected) {
            for (actual, field) in [
                (&job.external_id, "external_id"),
                (&job.title, "title"),
                (&job.company, "company"),
                (&job.url, "url"),
                (&job.description, "description"),
                (&job.location, "location"),
            ] {
                assert_eq!(actual, row[field].as_str().unwrap(), "{key} {field}");
            }
            assert_eq!(job.work_mode_hint.as_deref(), row["hint"].as_str(), "{key}");
            assert_eq!(
                job.fingerprint(),
                row["fingerprint"].as_str().unwrap(),
                "{key}"
            );
            let tags: Vec<&str> = row["tags"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .collect();
            assert_eq!(job.tags, tags, "{key} tags");
            let published = job.published_at.map(|at| at.format("%Y-%m-%d").to_string());
            let expected_date = row["published_at"].as_str().map(|at| at[..10].to_owned());
            assert_eq!(published, expected_date, "{key} published_at");
        }
    }

    /// Serves preset pages; anything else, robots.txt included, is a 404.
    #[derive(Default)]
    pub struct Scripted {
        pub pages: HashMap<String, (u16, String)>,
        pub requested: RefCell<Vec<String>>,
        pub slept: RefCell<Vec<f64>>,
    }

    impl Scripted {
        pub fn page(mut self, url: &str, status: u16, body: &str) -> Self {
            self.pages.insert(url.to_owned(), (status, body.to_owned()));
            self
        }

        pub fn pages_requested(&self) -> Vec<String> {
            let requested = self.requested.borrow();
            requested
                .iter()
                .filter(|u| !u.ends_with("/robots.txt"))
                .cloned()
                .collect()
        }
    }

    impl Transport for &Scripted {
        fn get(&self, url: &str, _accept: &str) -> Result<RawResponse, String> {
            self.requested.borrow_mut().push(url.to_owned());
            let (status, body) = self.pages.get(url).cloned().unwrap_or((404, String::new()));
            Ok(RawResponse {
                status,
                body: body.into_bytes(),
                retry_after: None,
            })
        }

        fn sleep(&self, seconds: f64) {
            self.slept.borrow_mut().push(seconds);
        }

        fn now(&self) -> f64 {
            self.slept.borrow().iter().sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::http::HttpError;

    #[test]
    fn detail_outcomes_follow_the_closed_rules() {
        let gupy = include_str!("../../../tests/fixtures/gupy_closed.html");
        assert_eq!(
            gupy::outcome_of(Ok(gupy.as_bytes().to_vec())),
            DetailOutcome::Closed
        );
        let solides = solides::outcome_of(Err(HttpError::Status(404)));
        assert_eq!(solides, DetailOutcome::Closed);
        assert_eq!(
            gupy::outcome_of(Err(HttpError::Status(410))),
            DetailOutcome::Closed
        );
        for transient in [
            HttpError::Blocked,
            HttpError::Status(403),
            HttpError::Status(500),
            HttpError::Network("x".into()),
            HttpError::TooLarge,
        ] {
            assert!(
                matches!(
                    solides::outcome_of(Err(transient.clone())),
                    DetailOutcome::Failed(_)
                ),
                "{transient:?}"
            );
            assert!(
                matches!(
                    gupy::outcome_of(Err(transient.clone())),
                    DetailOutcome::Failed(_)
                ),
                "{transient:?}"
            );
        }
        // No JSON-LD on a 200 is a parse failure, not a closed ad.
        assert!(matches!(
            solides::outcome_of(Ok(b"<html></html>".to_vec())),
            DetailOutcome::Failed(_)
        ));
        assert!(matches!(
            gupy::outcome_of(Ok(b"<html></html>".to_vec())),
            DetailOutcome::Failed(_)
        ));
    }

    #[test]
    fn dates_read_like_python_fromisoformat() {
        let at = |raw: &str| parse_iso(raw).map(|d| d.to_rfc3339());
        assert_eq!(
            at("2026-10-31T15:01:04Z").unwrap(),
            "2026-10-31T15:01:04+00:00"
        );
        assert_eq!(at("2026-09-23").unwrap(), "2026-09-23T00:00:00-03:00");
        assert_eq!(
            at("2026-09-23T10:30:00").unwrap(),
            "2026-09-23T10:30:00-03:00"
        );
        assert_eq!(at("ontem"), None);
    }
}
