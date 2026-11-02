//! Gupy, as in `sources/gupy.py`: the search page embeds its jobs in `__NEXT_DATA__`.

use std::rc::Rc;
use std::sync::LazyLock;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;
use serde_json::Value;

use super::{DetailOutcome, JobDetail, Source, error_outcome, id_text, parse_iso, text};
use crate::crawler::http::{Client, GetOptions, HttpError, Transport};
use crate::crawler::job::{Job, clean_html};

const SEARCH: &str = "https://portal.gupy.io/job-search";
/// The portal serves 12 per page; one page per term is enough for a daily radar.
const PER_TERM: usize = 12;
/// `targets.py`: the Brazilian market names one role in several ways.
pub const TERMS: [&str; 15] = [
    "ai engineer",
    "engenheiro de ia",
    "machine learning engineer",
    "engenheiro de machine learning",
    "desenvolvedor senior",
    "engenheiro de software senior",
    "full stack senior",
    "desenvolvedor full stack",
    "tech lead",
    "lider tecnico",
    "engineering manager",
    "gerente de tecnologia",
    "engenheiro de dados",
    "data engineer",
    "engenheiro de dados senior",
];
/// Python's `urllib.parse.quote(term, safe="")`.
const TERM_ENCODE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'.')
    .remove(b'-')
    .remove(b'~');

static NEXT_DATA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<script[^>]*id=["']__NEXT_DATA__["'][^>]*>(.*?)</script>"#).unwrap()
});

fn next_data(html: &str) -> Result<Value, String> {
    let payload = NEXT_DATA
        .captures(html)
        .and_then(|found| found.get(1))
        .ok_or("página sem __NEXT_DATA__")?;
    serde_json::from_str(payload.as_str()).map_err(|_| "__NEXT_DATA__ inválido".to_owned())
}

fn parse_job(raw: &Value) -> Job {
    let city = text(raw, "city").trim();
    let state = text(raw, "state").trim();
    let mut job = Job::new(
        "gupy",
        &id_text(raw),
        text(raw, "name"),
        text(raw, "careerPageName"),
        Some(text(raw, "jobUrl"))
            .filter(|url| !url.is_empty())
            .unwrap_or(text(raw, "careerPageUrl")),
    )
    .description(text(raw, "description"))
    .location(&place(city, state, text(raw, "country")))
    .work_mode_hint(raw["workplaceType"].as_str());
    job.tags = raw["skills"]
        .as_array()
        .map(|skills| {
            skills
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    job.published_at = parse_iso(text(raw, "publishedDate"));
    job
}

fn place(city: &str, state: &str, country: &str) -> String {
    match (city.is_empty(), state.is_empty()) {
        (true, true) => country.to_owned(),
        (false, false) => format!("{city}, {state}"),
        _ => format!("{city}{state}"),
    }
}

/// A page without the job list means the shape changed, which must not read as "no jobs".
pub fn parse_search(html: &str) -> Result<Vec<Job>, String> {
    let data = next_data(html)?;
    let listing = &data["props"]["pageProps"]["initialJobList"];
    let items = match listing {
        Value::Array(items) => items,
        Value::Object(_) => listing["data"].as_array().map_or(&[][..], Vec::as_slice),
        _ => return Err("__NEXT_DATA__ sem initialJobList".to_owned()),
    };
    Ok(items
        .iter()
        .filter(|item| item.is_object())
        .map(parse_job)
        .collect())
}

/// `Ok(None)` is Gupy's closed ad: a page that answers 200 with no job in it.
pub fn parse_detail(html: &str) -> Result<Option<JobDetail>, String> {
    let data = next_data(html)?;
    let job = &data["props"]["pageProps"]["job"];
    if !job.is_object() {
        return Ok(None);
    }
    let description = [
        "description",
        "responsibilities",
        "prerequisites",
        "relevantExperiences",
    ]
    .iter()
    .map(|field| clean_html(text(job, field)))
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    if description.is_empty() {
        return Err("vaga sem descrição".to_owned());
    }
    let location = place(
        text(job, "addressCity").trim(),
        text(job, "addressStateShortName").trim(),
        text(job, "addressCountry").trim(),
    );
    Ok(Some(JobDetail {
        description,
        location,
        work_mode_hint: job["workplaceType"].as_str().map(str::to_owned),
        published_at: parse_iso(text(job, "publishedAt")),
    }))
}

pub fn outcome_of(result: Result<Vec<u8>, HttpError>) -> DetailOutcome {
    match result {
        Err(error) => error_outcome(error),
        Ok(body) => match parse_detail(&String::from_utf8_lossy(&body)) {
            Ok(Some(detail)) => DetailOutcome::Found(detail),
            Ok(None) => DetailOutcome::Closed,
            Err(reason) => DetailOutcome::Failed(reason),
        },
    }
}

pub struct Gupy<T: Transport> {
    client: Rc<Client<T>>,
}

impl<T: Transport> Gupy<T> {
    pub fn new(client: Rc<Client<T>>) -> Self {
        Self { client }
    }

    fn get_html(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        let options = GetOptions {
            accept: Some("text/html"),
            ..GetOptions::default()
        };
        self.client.get(url, options).map(|response| response.body)
    }
}

impl<T: Transport> Source for Gupy<T> {
    fn name(&self) -> &'static str {
        "gupy"
    }

    fn owns(&self, source: &str) -> bool {
        source == "gupy"
    }

    /// The search already brings the description, so `keep` saves no request here.
    /// A failed term leaves the others alone; the source fails only when all of them do.
    fn collect(
        &self,
        _keep: &dyn Fn(&Job) -> bool,
        progress: &dyn Fn(&str),
    ) -> Result<Vec<Job>, String> {
        let mut jobs = Vec::new();
        let mut last_error = None;
        let mut any_ok = false;
        for term in TERMS {
            progress(&format!("gupy: {term}"));
            // The term goes in the PATH: with "?term=" the portal answers 200 with an empty search.
            let url = format!("{SEARCH}/term={}", utf8_percent_encode(term, TERM_ENCODE));
            let page = self
                .get_html(&url)
                .map_err(|error| format!("{error:?}"))
                .and_then(|body| parse_search(&String::from_utf8_lossy(&body)));
            match page {
                Ok(found) => {
                    any_ok = true;
                    jobs.extend(found.into_iter().take(PER_TERM));
                }
                Err(reason) => last_error = Some(reason),
            }
        }
        match (any_ok, last_error) {
            (false, Some(reason)) => Err(reason),
            _ => Ok(jobs),
        }
    }

    fn fetch_detail(&self, url: &str) -> DetailOutcome {
        outcome_of(self.get_html(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::{Scripted, assert_jobs_match};

    #[test]
    fn parses_the_recorded_search_like_python() {
        let jobs = parse_search(include_str!("../../../tests/fixtures/gupy_search.html")).unwrap();
        assert_jobs_match(&jobs, "gupy");
        assert!(!jobs.is_empty());
    }

    #[test]
    fn a_page_without_the_payload_is_an_error() {
        assert!(parse_search("<html>nothing</html>").is_err());
        assert!(parse_search("<script id=\"__NEXT_DATA__\">{ broken</script>").is_err());
    }

    #[test]
    fn reads_a_job_page() {
        let detail = parse_detail(include_str!("../../../tests/fixtures/gupy_job.html"))
            .unwrap()
            .unwrap();
        assert!(detail.description.contains("Python"));
        assert!(detail.published_at.is_some());
        assert_eq!(detail.location, "São Paulo, SP");
        assert_eq!(detail.work_mode_hint.as_deref(), Some("remote"));
        assert!(detail.description.contains("PostgreSQL & AWS"));
        assert!(detail.description.contains("CLT"));
    }

    #[test]
    fn a_page_without_a_job_is_closed() {
        assert_eq!(
            parse_detail(include_str!("../../../tests/fixtures/gupy_closed.html")),
            Ok(None)
        );
        assert!(parse_detail("<html>no payload at all</html>").is_err());
    }

    #[test]
    fn a_job_with_no_text_is_a_failure_not_a_closed_ad() {
        let html = r#"<script id="__NEXT_DATA__">{"props":{"pageProps":{"job":{"id":1,"description":""}}}}</script>"#;
        assert!(parse_detail(html).is_err());
    }

    #[test]
    fn collect_searches_every_term_in_the_path_and_keeps_the_good_ones() {
        let page = include_str!("../../../tests/fixtures/gupy_search.html");
        let scripted = Scripted::default().page(
            "https://portal.gupy.io/job-search/term=data%20engineer",
            200,
            page,
        );
        let gupy = Gupy::new(Rc::new(Client::new(&scripted)));
        let jobs = gupy.collect(&|_| true, &|_| {}).unwrap();
        assert_eq!(jobs.len(), PER_TERM);
        assert_eq!(scripted.pages_requested().len(), TERMS.len());
    }

    #[test]
    fn collect_fails_only_when_every_term_fails() {
        let scripted = Scripted::default();
        let gupy = Gupy::new(Rc::new(Client::new(&scripted)));
        assert!(gupy.collect(&|_| true, &|_| {}).is_err());
    }
}
