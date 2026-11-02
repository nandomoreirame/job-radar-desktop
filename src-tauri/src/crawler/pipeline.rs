//! One crawl: collect, filter, score, route, then rescore what has no score.
//! Each job commits on its own, so closing the app loses only the job in flight.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::clock::Now;
use crate::crawler::filters::{
    FilterSettings, WorkMode, count_skill_hits, passes_hard_filters, work_mode_of,
};
use crate::crawler::job::Job;
use crate::crawler::scoring::score_job;
use crate::crawler::sources::{Collected, DetailOutcome, JobDetail, Source};
use crate::domain::{Stage, WorkModel};
use crate::identity::{target_from_title, url_key};
use crate::import::{
    BELOW_CUT, ImportReport, MAX_DESCRIPTION, MAX_TEXT, NewJob, base, bounded, date_or, valid,
};
use crate::sqlite_store::{KnownKeys, ScoreUpdate, SqliteStore, UnscoredCard};
use crate::store::Store;

pub const CLOSED_AD: &str = "anúncio encerrado";
/// Below this many pages one source's closed count proves nothing.
const BREAKER_MIN_PAGES: usize = 10;
/// Where errors that belong to no source go.
const STORE_ERRORS: &str = "banco";
const INVALID_JOB: &str = "dados inválidos";

#[derive(Debug, Clone)]
pub struct RunSettings {
    pub skills: BTreeSet<String>,
    pub min_score: u8,
    pub min_skills: usize,
    pub filters: FilterSettings,
    pub now: Now,
}

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceStats {
    pub found: usize,
    pub error: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunStats {
    pub found: usize,
    pub inbox: usize,
    pub discarded: usize,
    pub known: usize,
    pub rejected: usize,
    pub rejected_by_reason: BTreeMap<String, usize>,
    pub rescored: usize,
    pub rescore_closed: usize,
    pub rescore_failed: usize,
    pub rescore_skipped: usize,
    pub sources: BTreeMap<String, SourceStats>,
}

impl RunStats {
    fn note_error(&mut self, source: &str, error: String) {
        let slot = &mut self.sources.entry(source.to_owned()).or_default().error;
        *slot = Some(match slot.take() {
            Some(previous) => format!("{previous}; {error}"),
            None => error,
        });
    }
}

pub fn run_crawl(
    store: &SqliteStore,
    sources: &[&dyn Source],
    settings: &RunSettings,
    progress: &dyn Fn(&str),
) -> RunStats {
    let mut stats = RunStats::default();
    collect_new_jobs(store, sources, settings, progress, &mut stats);
    rescore(store, sources, settings, progress, &mut stats);
    stats
}

fn today(settings: &RunSettings) -> String {
    settings.now.format("%Y-%m-%d").to_string()
}

fn work_model(job: &Job) -> Option<WorkModel> {
    match work_mode_of(job) {
        WorkMode::Remote => Some(WorkModel::Remote),
        WorkMode::Hybrid => Some(WorkModel::Hybrid),
        WorkMode::Onsite => Some(WorkModel::Onsite),
        WorkMode::Unknown => None,
    }
}

fn scored(job: &Job, settings: &RunSettings) -> u8 {
    let hits = count_skill_hits(job, &settings.skills).len();
    score_job(job, hits, settings.now)
}

/// The first variant that passes, else the first one, as Python's `select_variant`:
/// Gupy publishes one job per city, and the first city may be the one that fails.
fn select_variant(group: Vec<Job>, settings: &RunSettings) -> (Job, Result<(), String>) {
    let check = |job: &Job| {
        passes_hard_filters(
            job,
            &settings.skills,
            settings.min_skills,
            &settings.filters,
            settings.now,
        )
    };
    let passing = group.iter().position(|job| check(job).is_ok()).unwrap_or(0);
    let job = group
        .into_iter()
        .nth(passing)
        .expect("a group holds at least one job");
    let verdict = check(&job);
    (job, verdict)
}

fn new_job(job: &Job, score: u8, settings: &RunSettings) -> NewJob {
    let today = today(settings);
    let found = job
        .published_at
        .map(|at| at.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let url = job.url.trim();
    let mut card = base(
        &job.fingerprint(),
        &job.source,
        &job.title,
        &job.company,
        url,
        &date_or(&found, &today),
    );
    card.score = Some(score);
    card.work_model = work_model(job);
    card.location = Some(bounded(&job.location, MAX_TEXT)).filter(|place| !place.is_empty());
    NewJob {
        card,
        url_key: url_key(url),
        description: bounded(&job.description, MAX_DESCRIPTION),
        discard: (score < settings.min_score).then(|| (BELOW_CUT.to_owned(), today)),
    }
}

fn collect_new_jobs(
    store: &SqliteStore,
    sources: &[&dyn Source],
    settings: &RunSettings,
    progress: &dyn Fn(&str),
    stats: &mut RunStats,
) {
    let keep = |job: &Job| {
        passes_hard_filters(job, &settings.skills, 0, &settings.filters, settings.now).is_ok()
    };
    let mut groups: Vec<Vec<Job>> = Vec::new();
    let mut group_of: HashMap<String, usize> = HashMap::new();
    for source in sources {
        let entry = stats.sources.entry(source.name().to_owned()).or_default();
        match source.collect(&keep, progress) {
            Ok(Collected { jobs, warnings }) => {
                entry.found = jobs.len();
                stats.found += jobs.len();
                if !warnings.is_empty() {
                    entry.error = Some(warnings.join("; "));
                }
                for job in jobs {
                    let next = groups.len();
                    let at = *group_of.entry(job.fingerprint()).or_insert(next);
                    if at == next {
                        groups.push(Vec::new());
                    }
                    groups[at].push(job);
                }
            }
            Err(error) => entry.error = Some(error),
        }
    }
    let mut known = match store.known_keys() {
        Ok(known) => known,
        Err(error) => {
            stats.note_error(STORE_ERRORS, error.detail());
            return;
        }
    };
    let mut broken = BTreeSet::new();
    for group in groups {
        let (job, verdict) = select_variant(group, settings);
        if broken.contains(&job.source) {
            continue;
        }
        if let Err(reason) = verdict {
            count_rejected(stats, &reason);
            continue;
        }
        if !valid(&job.fingerprint(), &job.title, job.url.trim()) {
            count_rejected(stats, INVALID_JOB);
            continue;
        }
        if !remember_new(&job, &mut known) {
            stats.known += 1;
            continue;
        }
        let new = new_job(&job, scored(&job, settings), settings);
        let mut report = ImportReport::default();
        match store.insert_new_jobs(vec![new], &mut report) {
            Ok(()) => {
                stats.inbox += report.inbox;
                stats.discarded += report.discarded;
                stats.known += report.duplicates + report.dismissed;
            }
            // A store error ends this source, never the run.
            Err(error) => {
                stats.note_error(&job.source, error.detail());
                broken.insert(job.source);
            }
        }
    }
}

fn count_rejected(stats: &mut RunStats, reason: &str) {
    stats.rejected += 1;
    let key = reason.split('(').next().unwrap_or(reason).trim().to_owned();
    *stats.rejected_by_reason.entry(key).or_default() += 1;
}

/// True when the job is new; either way a later duplicate in this run is known.
fn remember_new(job: &Job, known: &mut KnownKeys) -> bool {
    let key = url_key(job.url.trim());
    let seen = known.ids.contains(&job.fingerprint()) || known.urls.contains(&key);
    known.ids.insert(job.fingerprint());
    if !key.is_empty() {
        known.urls.insert(key);
    }
    !seen
}

fn hint_of(model: Option<WorkModel>) -> Option<&'static str> {
    match model? {
        WorkModel::Remote => Some("remote"),
        WorkModel::Hybrid => Some("hybrid"),
        WorkModel::Onsite => Some("onsite"),
    }
}

fn job_of(card: &UnscoredCard) -> Job {
    let url = card.job_url.as_deref().unwrap_or("");
    Job::new(&card.source, "", &card.title, &card.company, url)
        .description(&card.description)
        .location(card.location.as_deref().unwrap_or(""))
        .work_mode_hint(hint_of(card.work_model))
}

fn apply(
    store: &SqliteStore,
    card: &UnscoredCard,
    job: &Job,
    stats: &mut RunStats,
    settings: &RunSettings,
    text: String,
) {
    let update = ScoreUpdate {
        score: scored(job, settings),
        target: target_from_title(&card.title),
        work_model: work_model(job),
        location: Some(bounded(&job.location, MAX_TEXT)).filter(|place| !place.is_empty()),
        description: text,
    };
    match store.apply_score(&card.id, &update) {
        Ok(()) => stats.rescored += 1,
        Err(error) => {
            stats.rescore_failed += 1;
            stats.note_error(STORE_ERRORS, error.detail());
        }
    }
}

fn apply_detail(
    store: &SqliteStore,
    card: &UnscoredCard,
    detail: JobDetail,
    stats: &mut RunStats,
    settings: &RunSettings,
) {
    let description = bounded(&detail.description, MAX_DESCRIPTION);
    let mut job = job_of(card).description(&description);
    if !detail.location.is_empty() {
        job = job.location(&detail.location);
    }
    if detail.work_mode_hint.is_some() {
        job = job.work_mode_hint(detail.work_mode_hint.as_deref());
    }
    job.published_at = detail.published_at;
    apply(store, card, &job, stats, settings, description);
}

/// Scores what has none. A low score here stays where it is: only a closed ad leaves.
fn rescore(
    store: &SqliteStore,
    sources: &[&dyn Source],
    settings: &RunSettings,
    progress: &dyn Fn(&str),
    stats: &mut RunStats,
) {
    let pending = store.unscored_cards().and_then(|mut cards| {
        cards.extend(store.unscored_discarded()?);
        Ok(cards)
    });
    let pending = match pending {
        Ok(pending) => pending,
        Err(error) => {
            stats.note_error(STORE_ERRORS, error.detail());
            return;
        }
    };
    let mut by_source: Vec<Vec<&UnscoredCard>> = vec![Vec::new(); sources.len()];
    for card in &pending {
        if !card.description.is_empty() {
            apply(store, card, &job_of(card), stats, settings, String::new());
        } else if let Some(at) = sources.iter().position(|source| {
            source.owns(&card.source)
                && card
                    .job_url
                    .as_deref()
                    .is_none_or(|url| source.accepts_url(url))
        }) {
            by_source[at].push(card);
        } else {
            stats.rescore_skipped += 1;
        }
    }
    for (source, cards) in sources.iter().zip(by_source) {
        fetch_pages(store, *source, &cards, settings, progress, stats);
    }
}

fn fetch_pages(
    store: &SqliteStore,
    source: &dyn Source,
    cards: &[&UnscoredCard],
    settings: &RunSettings,
    progress: &dyn Fn(&str),
    stats: &mut RunStats,
) {
    let mut pages = 0;
    let mut closed = Vec::new();
    for (done, card) in cards.iter().enumerate() {
        progress(&format!("{}: {}/{}", source.name(), done + 1, cards.len()));
        let Some(url) = card.job_url.as_deref() else {
            stats.rescore_failed += 1;
            continue;
        };
        pages += 1;
        match source.fetch_detail(url) {
            DetailOutcome::Found(detail) => apply_detail(store, card, detail, stats, settings),
            DetailOutcome::Closed => closed.push(*card),
            DetailOutcome::Failed(_) => stats.rescore_failed += 1,
        }
    }
    // Most of a source "closed" at once is a site that changed, not a wave of closed ads.
    if pages >= BREAKER_MIN_PAGES && closed.len() * 2 > pages {
        stats.note_error(
            source.name(),
            format!(
                "{} de {pages} páginas parecem encerradas; nada foi descartado",
                closed.len()
            ),
        );
        return;
    }
    let today = today(settings);
    for card in closed.iter().filter(|card| card.stage == Stage::Inbox) {
        match store.discard_card(&card.id, CLOSED_AD, &today) {
            Ok(()) => stats.rescore_closed += 1,
            Err(error) => {
                stats.rescore_failed += 1;
                stats.note_error(STORE_ERRORS, error.detail());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::job::Job;
    use crate::crawler::sources::{DetailOutcome, JobDetail, Source};
    use crate::domain::Stage;
    use crate::import::{ImportReport, NewJob};
    use crate::sqlite_store::SqliteStore;
    use crate::store::Store;
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashMap};

    struct FakeSource {
        name: &'static str,
        jobs: Result<Vec<Job>, String>,
        warnings: Vec<String>,
        details: HashMap<String, DetailOutcome>,
        fetched: RefCell<Vec<String>>,
    }
    impl FakeSource {
        fn new(name: &'static str, jobs: Vec<Job>) -> Self {
            Self {
                name,
                jobs: Ok(jobs),
                warnings: Vec::new(),
                details: HashMap::new(),
                fetched: RefCell::new(Vec::new()),
            }
        }
    }
    impl Source for FakeSource {
        fn name(&self) -> &'static str {
            self.name
        }
        fn owns(&self, source: &str) -> bool {
            source == self.name
        }
        fn accepts_url(&self, url: &str) -> bool {
            !url.contains("localhost")
        }
        fn collect(
            &self,
            _keep: &dyn Fn(&Job) -> bool,
            _progress: &dyn Fn(&str),
        ) -> Result<Collected, String> {
            self.jobs.clone().map(|jobs| Collected {
                jobs,
                warnings: self.warnings.clone(),
            })
        }
        fn fetch_detail(&self, url: &str) -> DetailOutcome {
            self.fetched.borrow_mut().push(url.to_owned());
            self.details
                .get(url)
                .cloned()
                .unwrap_or(DetailOutcome::Failed("no page".into()))
        }
    }

    fn skills() -> BTreeSet<String> {
        [
            "python",
            "aws",
            "docker",
            "typescript",
            "react",
            "rust",
            "sql",
            "linux",
            "git",
            "fastapi",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect()
    }
    fn setup() -> RunSettings {
        RunSettings {
            skills: skills(),
            min_score: 70,
            min_skills: 3,
            filters: Default::default(),
            now: crate::clock::fixed_now(),
        }
    }
    fn job(title: &str, company: &str, url: &str, description: &str) -> Job {
        Job::new("gupy", "1", title, company, url)
            .description(description)
            .work_mode_hint(Some("remote"))
    }
    const STRONG: &str = "Python AWS Docker TypeScript React Rust SQL Linux Git FastAPI";
    const WEAK: &str = "Python AWS Docker";

    /// A stored card with no score, as the import leaves it.
    fn imported(store: &SqliteStore, id: &str, source: &str, url: &str, description: &str) {
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.id = id.into();
        card.source = source.into();
        card.score = None;
        card.target = None;
        card.work_model = None;
        card.stage = Stage::Inbox;
        card.outcome = None;
        card.job_url = Some(url.into());
        let new = NewJob {
            card,
            url_key: crate::identity::url_key(url),
            description: description.into(),
            discard: None,
        };
        store
            .insert_new_jobs(vec![new], &mut ImportReport::default())
            .unwrap();
    }

    #[test]
    fn routes_new_jobs_and_skips_known_ones() {
        let store = SqliteStore::open_in_memory().unwrap();
        let known = job(
            "Conhecida Sênior",
            "Acme",
            "https://a.example/known",
            STRONG,
        );
        imported(
            &store,
            &known.fingerprint(),
            "gupy",
            "https://a.example/known",
            STRONG,
        );
        let source = FakeSource::new(
            "gupy",
            vec![
                job(
                    "Engenheiro de IA Sênior",
                    "Beta",
                    "https://a.example/high",
                    STRONG,
                ),
                job(
                    "Desenvolvedor Sênior",
                    "Gama",
                    "https://a.example/low",
                    WEAK,
                ),
                job(
                    "Desenvolvedor Pleno",
                    "Delta",
                    "https://a.example/rej",
                    STRONG,
                ),
                known,
                job(
                    "Outro título Sênior",
                    "Acme",
                    "https://www.a.example/known/",
                    STRONG,
                ),
            ],
        );
        let sources: Vec<&dyn Source> = vec![&source];
        let stats = run_crawl(&store, &sources, &setup(), &|_| {});
        let inbox = store.list_cards().unwrap();
        let high = inbox
            .iter()
            .find(|c| c.job_url.as_deref() == Some("https://a.example/high"))
            .unwrap();
        assert!(high.score.unwrap() >= 70 && high.unread && high.stage == Stage::Inbox);
        let discarded = store.list_discarded().unwrap();
        assert_eq!(discarded.len(), 1);
        assert_eq!(discarded[0].reason, crate::import::BELOW_CUT);
        assert!(discarded[0].score.unwrap() < 70);
        assert_eq!(stats.rejected_by_reason["senioridade abaixo de sênior"], 1);
        assert_eq!(stats.known, 2);
        assert_eq!((stats.inbox, stats.discarded), (1, 1));
    }

    #[test]
    fn too_few_skills_are_rejected_and_not_stored() {
        let store = SqliteStore::open_in_memory().unwrap();
        let source = FakeSource::new(
            "gupy",
            vec![job(
                "Desenvolvedor Sênior",
                "Gama",
                "https://a.example/few",
                "Python AWS",
            )],
        );
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!((stats.rejected, stats.inbox, stats.discarded), (1, 0, 0));
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn a_link_outside_the_source_is_never_requested() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(
            &store,
            "aaaaaaaaaaaaaaaa",
            "gupy",
            "http://localhost:9/x",
            "",
        );
        let source = FakeSource::new("gupy", vec![]);
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert!(source.fetched.borrow().is_empty());
        assert_eq!(stats.rescore_skipped, 1);
        assert_eq!(store.list_cards().unwrap()[0].score, None);
    }

    #[test]
    fn a_partial_failure_keeps_the_jobs_and_marks_the_source() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut source = FakeSource::new(
            "gupy",
            vec![job(
                "Engenheiro de IA Sênior",
                "Beta",
                "https://a.example/1",
                STRONG,
            )],
        );
        source.warnings = vec!["2 de 15 buscas falharam".into()];
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.inbox, 1);
        assert_eq!(
            stats.sources["gupy"].error.as_deref(),
            Some("2 de 15 buscas falharam")
        );
    }

    #[test]
    fn a_job_with_an_unsafe_link_is_rejected() {
        let store = SqliteStore::open_in_memory().unwrap();
        let source = FakeSource::new(
            "gupy",
            vec![job(
                "Engenheiro de IA Sênior",
                "Beta",
                "javascript:alert(1)",
                STRONG,
            )],
        );
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.rejected_by_reason[INVALID_JOB], 1);
        assert!(store.list_cards().unwrap().is_empty());
    }

    #[test]
    fn a_new_job_uses_the_first_passing_variant_and_one_card() {
        let store = SqliteStore::open_in_memory().unwrap();
        let source = FakeSource::new(
            "gupy",
            vec![
                job(
                    "Engenheiro de IA Sênior",
                    "Beta",
                    "https://a.example/recife",
                    STRONG,
                )
                .work_mode_hint(Some("onsite"))
                .location("Recife, PE"),
                job(
                    "Engenheiro de IA Sênior",
                    "Beta",
                    "https://a.example/remoto",
                    STRONG,
                ),
            ],
        );
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.inbox, 1);
        assert_eq!(stats.rejected, 0);
        let cards = store.list_cards().unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(
            cards[0].job_url.as_deref(),
            Some("https://a.example/remoto")
        );
    }

    #[test]
    fn a_closed_page_only_discards_from_the_inbox_and_a_found_page_fills_the_card() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(
            &store,
            "aaaaaaaaaaaaaaaa",
            "gupy",
            "https://a.example/1",
            "",
        );
        let mut source = FakeSource::new("gupy", vec![]);
        let detail = JobDetail {
            description: STRONG.into(),
            ..Default::default()
        };
        source
            .details
            .insert("https://a.example/1".into(), DetailOutcome::Found(detail));
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(
            (stats.rescored, stats.rescore_closed, stats.rescore_failed),
            (1, 0, 0)
        );
        store
            .discard_card("aaaaaaaaaaaaaaaa", "x", "2026-11-02")
            .unwrap();
        assert_eq!(store.list_discarded().unwrap()[0].description, STRONG);
    }

    #[test]
    fn a_deleted_job_never_returns_from_a_crawl() {
        let store = SqliteStore::open_in_memory().unwrap();
        let weak = job(
            "Desenvolvedor Sênior",
            "Gama",
            "https://a.example/low",
            WEAK,
        );
        let sources_one = FakeSource::new("gupy", vec![weak.clone()]);
        run_crawl(&store, &[&sources_one as &dyn Source], &setup(), &|_| {});
        store
            .delete_card(&weak.fingerprint(), "2026-11-02")
            .unwrap();
        let again = FakeSource::new("gupy", vec![weak]);
        let stats = run_crawl(&store, &[&again as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.discarded, 0);
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn a_failing_source_does_not_stop_the_others() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut broken = FakeSource::new("solides", vec![]);
        broken.jobs = Err("payload ausente".into());
        let good = FakeSource::new(
            "gupy",
            vec![job(
                "Engenheiro de IA Sênior",
                "Beta",
                "https://a.example/1",
                STRONG,
            )],
        );
        let stats = run_crawl(&store, &[&broken as &dyn Source, &good], &setup(), &|_| {});
        assert_eq!(stats.inbox, 1);
        assert_eq!(
            stats.sources["solides"].error.as_deref(),
            Some("payload ausente")
        );
        assert_eq!(stats.sources["gupy"].error, None);
    }

    #[test]
    fn rescoring_keeps_user_edits_and_stage() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(
            &store,
            "aaaaaaaaaaaaaaaa",
            "gupy",
            "https://a.example/1",
            "",
        );
        store
            .move_card("aaaaaaaaaaaaaaaa", Stage::Applied, None)
            .unwrap();
        let details = crate::domain::CardDetails {
            notes: "nota".into(),
            job_url: Some("https://a.example/1".into()),
            ..Default::default()
        };
        store.update_card("aaaaaaaaaaaaaaaa", details).unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        source.details.insert(
            "https://a.example/1".into(),
            DetailOutcome::Found(JobDetail {
                description: STRONG.into(),
                location: "Recife".into(),
                work_mode_hint: Some("remote".into()),
                published_at: None,
            }),
        );
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let card = store.list_cards().unwrap().remove(0);
        assert!(card.score.is_some());
        assert_eq!((card.stage, card.notes.as_str()), (Stage::Applied, "nota"));
        assert_eq!(card.location.as_deref(), Some("Recife"));
    }

    #[test]
    fn a_stored_description_needs_no_request() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(
            &store,
            "aaaaaaaaaaaaaaaa",
            "gupy",
            "https://a.example/1",
            STRONG,
        );
        let source = FakeSource::new("gupy", vec![]);
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert!(source.fetched.borrow().is_empty());
        assert!(store.list_cards().unwrap()[0].score.is_some());
    }

    #[test]
    fn a_low_rescore_stays_in_the_inbox() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(
            &store,
            "aaaaaaaaaaaaaaaa",
            "gupy",
            "https://a.example/1",
            "curto",
        );
        let source = FakeSource::new("gupy", vec![]);
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let card = store.list_cards().unwrap().remove(0);
        assert!(card.score.unwrap() < 70);
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn closed_discards_and_failures_keep() {
        let store = SqliteStore::open_in_memory().unwrap();
        for (id, url) in [
            ("aaaaaaaaaaaaaaaa", "https://a.example/closed"),
            ("bbbbbbbbbbbbbbbb", "https://a.example/fail"),
            ("cccccccccccccccc", "https://a.example/moved"),
        ] {
            imported(&store, id, "gupy", url, "");
        }
        imported(
            &store,
            "dddddddddddddddd",
            "lever",
            "https://a.example/unsupported",
            "",
        );
        store
            .move_card("cccccccccccccccc", Stage::Applied, None)
            .unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        source
            .details
            .insert("https://a.example/closed".into(), DetailOutcome::Closed);
        source
            .details
            .insert("https://a.example/moved".into(), DetailOutcome::Closed);
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let discarded = store.list_discarded().unwrap();
        assert_eq!(
            discarded.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
            ["aaaaaaaaaaaaaaaa"]
        );
        assert_eq!(discarded[0].reason, CLOSED_AD);
        let left: Vec<_> = store.list_cards().unwrap();
        assert_eq!(left.len(), 3);
        assert!(left.iter().all(|c| c.score.is_none()));
        assert_eq!(
            left.iter()
                .find(|c| c.id == "cccccccccccccccc")
                .unwrap()
                .stage,
            Stage::Applied
        );
        assert_eq!(stats.rescore_closed, 1);
        assert!(
            !source
                .fetched
                .borrow()
                .iter()
                .any(|u| u.contains("unsupported"))
        );
    }

    #[test]
    fn circuit_breaker_stops_a_mass_discard() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        for i in 0..12 {
            let id = format!("{i:016x}");
            let url = format!("https://a.example/{i}");
            imported(&store, &id, "gupy", &url, "");
            source.details.insert(
                url,
                if i < 11 {
                    DetailOutcome::Closed
                } else {
                    DetailOutcome::Found(JobDetail {
                        description: STRONG.into(),
                        ..Default::default()
                    })
                },
            );
        }
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert!(store.list_discarded().unwrap().is_empty());
        assert_eq!(stats.rescore_closed, 0);
        assert!(
            stats.sources["gupy"]
                .error
                .as_deref()
                .unwrap()
                .contains("encerrad")
        );
        assert!(
            store
                .list_cards()
                .unwrap()
                .iter()
                .any(|c| c.score.is_some())
        );
    }

    #[test]
    fn nine_closed_pages_are_below_the_breaker_minimum() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        for i in 0..9 {
            let id = format!("{i:016x}");
            let url = format!("https://a.example/{i}");
            imported(&store, &id, "gupy", &url, "");
            source.details.insert(url, DetailOutcome::Closed);
        }
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.rescore_closed, 9);
    }
}
