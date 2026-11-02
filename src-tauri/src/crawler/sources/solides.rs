//! Solides, as in `sources/solides.py`: the listing lives in the RSC payload and has
//! no description, so each survivor of the cheap filters gets its JSON-LD read.

use std::rc::Rc;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use super::{DetailOutcome, JobDetail, Source, error_outcome, id_text, parse_iso, text};
use crate::clock::Now;
use crate::crawler::filters::AFFIRMATIVE_TAG;
use crate::crawler::http::{Client, GetOptions, HttpError, Transport};
use crate::crawler::job::{Job, clean_html};

const BASE: &str = "https://vagas.solides.com.br";
const MAX_PAGES: u64 = 50;
const DELAY_BETWEEN_PAGES: f64 = 1.2;
const DELAY_BETWEEN_DETAILS: f64 = 0.8;
/// The end of the listing serves ads from 2023; applying to them burns the contact.
const MAX_AGE_DAYS: i64 = 45;
const PUSH: &str = "self.__next_f.push([1,";

static LD_JSON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<script[^>]*application/ld\+json[^>]*>(.*?)</script>").unwrap()
});

/// Joins the RSC pushes, each decoded as a real JSON string so escaped quotes survive.
pub fn rsc_blob(html: &str) -> String {
    let mut blob = String::new();
    for (at, _) in html.match_indices(PUSH) {
        let rest = &html[at + PUSH.len()..];
        if let Some(Ok(part)) = serde_json::Deserializer::from_str(rest)
            .into_iter::<String>()
            .next()
        {
            blob.push_str(&part);
        }
    }
    blob
}

/// The object opening at `start`, ignoring braces inside strings.
fn slice_json(text: &str, start: usize) -> &str {
    let (mut depth, mut in_string, mut escaped) = (0usize, false, false);
    for (offset, byte) in text.as_bytes()[start..].iter().enumerate() {
        if in_string {
            match byte {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[start..=start + offset];
                }
            }
            _ => {}
        }
    }
    ""
}

/// Python's truthiness, for `affirmative` (a list) and `pcdOnly` (a bool).
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Array(items) => !items.is_empty(),
        Value::String(raw) => !raw.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
        Value::Number(number) => number.as_f64() != Some(0.0),
    }
}

/// The URL is built from the id, never from `redirectLink`: the source sends a host
/// with no domain, and the company subdomain answers 200 with an empty page.
fn parse_listing(raw: &Value) -> Job {
    let id = id_text(raw);
    let place: Vec<&str> = [text(&raw["city"], "name"), text(&raw["state"], "code")]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    let hint = match text(raw, "jobType") {
        "remoto" => Some("remote"),
        "hibrido" => Some("hybrid"),
        "presencial" => Some("onsite"),
        _ => None,
    };
    let mut job = Job::new(
        "solides",
        &id,
        text(raw, "title"),
        text(raw, "companyName"),
        &format!("{BASE}/vaga/{id}"),
    )
    .location(&place.join(", "))
    .work_mode_hint(hint);
    for field in ["recruitmentContractType", "seniority"] {
        if let Some(items) = raw[field].as_array() {
            job.tags
                .extend(items.iter().map(|item| text(item, "name").to_owned()));
        }
    }
    if truthy(&raw["affirmative"]) || truthy(&raw["pcdOnly"]) {
        job.tags.push(AFFIRMATIVE_TAG.to_owned());
    }
    job.tags.retain(|tag| !tag.is_empty());
    job.published_at = parse_iso(text(raw, "createdAt"));
    job
}

/// False only when the source states the ad is old: no date is no proof of an old ad.
pub fn is_fresh(job: &Job, max_age_days: i64, now: Now) -> bool {
    job.published_at
        .is_none_or(|at| (now.date_naive() - at.date_naive()).num_days() <= max_age_days)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListingMeta {
    pub total_pages: Option<u64>,
}

/// A blob without the listing means the shape changed, which must not read as "no jobs".
pub fn listing_from_blob(blob: &str) -> Result<(ListingMeta, Vec<Job>), String> {
    let marker = blob
        .find("\"initialData\":")
        .ok_or("página sem initialData")?;
    let start = blob[marker..].find('{').ok_or("initialData sem objeto")? + marker;
    let data: Value = serde_json::from_str(slice_json(blob, start))
        .map_err(|_| "initialData inválido".to_owned())?;
    let meta = ListingMeta {
        total_pages: data["totalPages"].as_u64(),
    };
    let jobs = data["data"]
        .as_array()
        .map_or_else(Vec::new, |items| items.iter().map(parse_listing).collect());
    Ok((meta, jobs))
}

/// The `JobPosting` description as plain text, or empty when the page has none.
pub fn parse_jobposting(html: &str) -> String {
    LD_JSON
        .captures(html)
        .and_then(|found| serde_json::from_str::<Value>(found.get(1)?.as_str()).ok())
        .filter(|data| data["@type"] == "JobPosting")
        .map(|data| clean_html(text(&data, "description")))
        .unwrap_or_default()
}

/// `validThrough` is ignored on purpose: only 404 or 410 closes a Solides job.
pub fn outcome_of(result: Result<Vec<u8>, HttpError>) -> DetailOutcome {
    match result {
        Err(error) => error_outcome(error),
        Ok(body) => {
            let description = parse_jobposting(&String::from_utf8_lossy(&body));
            if description.is_empty() {
                return DetailOutcome::Failed("página sem JobPosting".to_owned());
            }
            DetailOutcome::Found(JobDetail {
                description,
                ..JobDetail::default()
            })
        }
    }
}

pub struct Solides<T: Transport> {
    client: Rc<Client<T>>,
    now: Now,
}

impl<T: Transport> Solides<T> {
    pub fn new(client: Rc<Client<T>>, now: Now) -> Self {
        Self { client, now }
    }

    /// Solides answers bursts with 403, so 403 and 429 are retried at a slow, fixed pace.
    fn get_html(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        let options = GetOptions {
            accept: Some("text/html"),
            retry_statuses: &[403, 429],
            fixed_backoff: Some(5.0),
            retries: Some(2),
        };
        self.client.get(url, options).map(|response| response.body)
    }

    fn list_fresh(&self, progress: &dyn Fn(&str)) -> Result<Vec<Job>, String> {
        let mut found = Vec::new();
        for page in 1..=MAX_PAGES {
            progress(&format!("solides: página {page}"));
            let url = format!("{BASE}/vagas/area/tecnologia/todas?seniorities=senior&page={page}");
            let listing = self
                .get_html(&url)
                .map_err(|error| format!("{error:?}"))
                .and_then(|body| listing_from_blob(&rsc_blob(&String::from_utf8_lossy(&body))));
            let (meta, jobs) = match listing {
                Ok(listing) => listing,
                // Later pages failing keep what the earlier ones found, as in Python.
                Err(reason) if page == 1 => return Err(reason),
                Err(_) => break,
            };
            let before = found.len();
            let total = jobs.len();
            found.extend(
                jobs.into_iter()
                    .filter(|job| is_fresh(job, MAX_AGE_DAYS, self.now)),
            );
            // Newest first: once a whole page is out of the window, the next ones are too.
            if total == 0 || found.len() == before || page >= meta.total_pages.unwrap_or(1) {
                break;
            }
            self.client.pause(DELAY_BETWEEN_PAGES);
        }
        Ok(found)
    }
}

impl<T: Transport> Source for Solides<T> {
    fn name(&self) -> &'static str {
        "solides"
    }

    fn owns(&self, source: &str) -> bool {
        source == "solides"
    }

    /// A failed detail keeps the job with no description, as in Python: with too few skills
    /// it is rejected and not stored, so the next crawl tries it again.
    fn collect(
        &self,
        keep: &dyn Fn(&Job) -> bool,
        progress: &dyn Fn(&str),
    ) -> Result<Vec<Job>, String> {
        let mut detailed = Vec::new();
        for mut job in self.list_fresh(progress)? {
            if !keep(&job) {
                continue;
            }
            if let DetailOutcome::Found(detail) = self.fetch_detail(&job.url) {
                job.description = detail.description;
            }
            detailed.push(job);
            self.client.pause(DELAY_BETWEEN_DETAILS);
        }
        Ok(detailed)
    }

    fn fetch_detail(&self, url: &str) -> DetailOutcome {
        outcome_of(self.get_html(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::{Scripted, assert_jobs_match, vectors};

    fn page_html(listing: &str) -> String {
        let blob = serde_json::to_string(&format!("\"initialData\":{listing}")).unwrap();
        format!("<script>self.__next_f.push([1,{blob}])</script>")
    }

    fn page_url(page: u32) -> String {
        format!("{BASE}/vagas/area/tecnologia/todas?seniorities=senior&page={page}")
    }

    #[test]
    fn parses_the_recorded_pages_like_python() {
        let listing = include_str!("../../../tests/fixtures/solides_listing.json");
        let (meta, jobs) = listing_from_blob(&format!("\"initialData\":{listing}")).unwrap();
        assert_jobs_match(&jobs, "solides");
        assert_eq!(
            meta.total_pages,
            vectors()["solides_meta"]["totalPages"].as_u64()
        );
        let posting = parse_jobposting(include_str!(
            "../../../tests/fixtures/solides_jobposting.html"
        ));
        assert_eq!(posting, vectors()["solides_posting"].as_str().unwrap());
    }

    #[test]
    fn keeps_a_job_without_a_date_and_drops_an_old_one() {
        let mut job = crate::crawler::job::Job::new("solides", "1", "Dev", "Acme", "https://x");
        assert!(is_fresh(&job, 45, crate::clock::fixed_now()));
        job.published_at =
            Some(chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00+00:00").unwrap());
        assert!(!is_fresh(&job, 45, crate::clock::fixed_now()));
    }

    #[test]
    fn decodes_pushes_with_escaped_quotes() {
        let html = r#"<script>self.__next_f.push([1,"a\"b"])</script><script>self.__next_f.push([1,"c"])</script>"#;
        assert_eq!(rsc_blob(html), "a\"bc");
    }

    #[test]
    fn a_blob_without_the_listing_is_an_error() {
        assert!(listing_from_blob("payload sem dados").is_err());
        assert!(listing_from_blob("\"initialData\":{ broken").is_err());
    }

    #[test]
    fn braces_and_escaped_quotes_in_strings_do_not_end_the_listing() {
        let blob = r#""initialData":{"note":"a\"}","data":[{"id":7,"title":"b{"}],"totalPages":1} trailing"#;
        let (meta, jobs) = listing_from_blob(blob).unwrap();
        assert_eq!(meta.total_pages, Some(1));
        assert_eq!(jobs[0].title, "b{");
    }

    #[test]
    fn a_later_page_failure_keeps_what_was_found() {
        let mut listing: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/solides_listing.json"))
                .unwrap();
        listing["totalPages"] = 3.into();
        let scripted =
            Scripted::default().page(&page_url(1), 200, &page_html(&listing.to_string()));
        let solides = Solides::new(Rc::new(Client::new(&scripted)), crate::clock::fixed_now());
        assert_eq!(solides.collect(&|_| true, &|_| {}).unwrap().len(), 3);
    }

    #[test]
    fn marks_a_job_the_source_declares_affirmative() {
        let raw = serde_json::json!({"id": 1, "affirmative": [{"id": 1, "name": "Mulheres"}]});
        assert!(
            parse_listing(&raw)
                .tags
                .contains(&AFFIRMATIVE_TAG.to_owned())
        );
        let raw = serde_json::json!({"id": 1, "affirmative": [], "pcdOnly": false});
        assert!(parse_listing(&raw).tags.is_empty());
    }

    #[test]
    fn collect_details_only_what_keep_approves_and_stops_at_the_last_page() {
        let mut listing: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/solides_listing.json"))
                .unwrap();
        listing["totalPages"] = 2.into();
        let page = page_html(&listing.to_string());
        let posting = include_str!("../../../tests/fixtures/solides_jobposting.html");
        let scripted = Scripted::default()
            .page(&page_url(1), 200, &page)
            .page(&page_url(2), 200, &page)
            .page(&format!("{BASE}/vaga/926052"), 200, posting);
        let solides = Solides::new(Rc::new(Client::new(&scripted)), crate::clock::fixed_now());
        let jobs = solides
            .collect(&|job| job.external_id == "926052", &|_| {})
            .unwrap();
        assert_eq!(jobs.len(), 2);
        assert!(jobs[0].description.contains("Desenvolvedor de Hardware"));
        let requested = scripted.pages_requested();
        assert_eq!(
            requested
                .iter()
                .filter(|url| url.contains("/vaga/"))
                .count(),
            2
        );
        assert_eq!(
            requested.iter().filter(|url| url.contains("page=")).count(),
            2
        );
        let slept = scripted.slept.borrow();
        assert!(slept.contains(&DELAY_BETWEEN_PAGES) && slept.contains(&DELAY_BETWEEN_DETAILS));
    }

    #[test]
    fn collect_stops_at_a_page_with_only_old_ads() {
        let mut listing: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/solides_listing.json"))
                .unwrap();
        for item in listing["data"].as_array_mut().unwrap() {
            item["createdAt"] = "2023-01-01".into();
        }
        let scripted =
            Scripted::default().page(&page_url(1), 200, &page_html(&listing.to_string()));
        let solides = Solides::new(Rc::new(Client::new(&scripted)), crate::clock::fixed_now());
        assert_eq!(solides.collect(&|_| true, &|_| {}).unwrap(), vec![]);
        assert_eq!(scripted.pages_requested().len(), 1);
    }

    #[test]
    fn a_first_page_failure_is_a_source_error() {
        let scripted = Scripted::default();
        let solides = Solides::new(Rc::new(Client::new(&scripted)), crate::clock::fixed_now());
        assert!(solides.collect(&|_| true, &|_| {}).is_err());
    }
}
