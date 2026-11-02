//! The job shape shared by every source, as in `job_radar/models.py`.

use std::sync::LazyLock;

use regex::Regex;

use crate::clock::Now;
use crate::crawler::identity::job_fingerprint;

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// Plain text out of a source's HTML: filters need text, not markup.
pub fn clean_html(raw: &str) -> String {
    let text = TAG.replace_all(raw, " ");
    let text = html_escape::decode_html_entities(&text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Job {
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub company: String,
    pub url: String,
    pub description: String,
    pub location: String,
    pub work_mode_hint: Option<String>,
    pub tags: Vec<String>,
    pub published_at: Option<Now>,
}

impl Job {
    pub fn new(source: &str, external_id: &str, title: &str, company: &str, url: &str) -> Self {
        Self {
            source: source.to_owned(),
            external_id: external_id.to_owned(),
            title: title.trim().to_owned(),
            company: company.trim().to_owned(),
            url: url.to_owned(),
            ..Self::default()
        }
    }

    pub fn description(mut self, raw: &str) -> Self {
        self.description = clean_html(raw);
        self
    }

    pub fn location(mut self, value: &str) -> Self {
        self.location = value.trim().to_owned();
        self
    }

    pub fn work_mode_hint(mut self, value: Option<&str>) -> Self {
        self.work_mode_hint = value.map(str::to_owned);
        self
    }

    pub fn fingerprint(&self) -> String {
        job_fingerprint(&self.company, &self.title)
    }

    /// The one text the filters search, case folded like Python's `casefold`.
    pub fn haystack(&self) -> String {
        let tags = self.tags.join(" ");
        let parts = [
            self.title.as_str(),
            &self.company,
            &self.description,
            &tags,
            &self.location,
        ];
        let joined = parts
            .iter()
            .filter(|p| !p.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        caseless::default_case_fold_str(&joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn cleans_html_like_python() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/parity/filters.json")).unwrap();
        for row in v["clean_html"].as_array().unwrap() {
            assert_eq!(
                clean_html(row[0].as_str().unwrap()),
                row[1].as_str().unwrap(),
                "{row}"
            );
        }
    }
}
