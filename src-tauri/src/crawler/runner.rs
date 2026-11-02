//! Starts a crawl on a worker thread. One at a time; progress goes out as events.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::clock::Now;
use crate::crawler::config::{ConfigError, CrawlSetup};
use crate::crawler::http::{Client, UreqTransport};
use crate::crawler::pipeline::{RunSettings, RunStats, run_crawl};
use crate::crawler::sources::Source;
use crate::crawler::sources::gupy::Gupy;
use crate::crawler::sources::solides::Solides;
use crate::sqlite_store::SqliteStore;
use crate::store::StoreError;

#[derive(Debug, Default, Clone)]
pub struct RunGuard(Arc<AtomicBool>);

/// Clears the flag on drop, including when the worker thread panics.
pub struct RunTicket(Arc<AtomicBool>);

impl RunGuard {
    pub fn try_start(&self) -> Result<RunTicket, &'static str> {
        if self.0.swap(true, Ordering::SeqCst) {
            return Err("already_running");
        }
        Ok(RunTicket(Arc::clone(&self.0)))
    }

    pub fn is_running(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

impl Drop for RunTicket {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

pub fn config_error_code(error: &ConfigError) -> String {
    match error {
        ConfigError::Missing => "config_missing".into(),
        ConfigError::Unreadable(_) => "config_unreadable".into(),
        ConfigError::Invalid(field) => format!("config_invalid:{field}"),
        ConfigError::ProfileUnreadable => "profile_unreadable".into(),
        ConfigError::NoSkills => "profile_no_skills".into(),
    }
}

/// Records the run around the pipeline. A run with no finish reads as interrupted.
pub fn record_run(
    store: &SqliteStore,
    sources: &[&dyn Source],
    setup: &CrawlSetup,
    now: Now,
    progress: &dyn Fn(&str),
) -> Result<RunStats, StoreError> {
    let id = store.start_run(&now.to_rfc3339())?;
    let settings = RunSettings {
        skills: setup.skills.clone(),
        min_score: setup.config.min_score,
        min_skills: setup.config.min_skills,
        filters: setup.config.filters.clone(),
        now,
    };
    let stats = run_crawl(store, sources, &settings, progress);
    let json = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_owned());
    store.finish_run(id, &crate::clock::now().to_rfc3339(), &json)?;
    Ok(stats)
}

/// The connectors hold an `Rc`, so they are built inside the worker thread.
pub fn crawl_with_real_sources(
    store: &SqliteStore,
    setup: &CrawlSetup,
    now: Now,
    progress: &dyn Fn(&str),
) -> Result<RunStats, StoreError> {
    let client = Rc::new(Client::new(UreqTransport::default()));
    let gupy = Gupy::new(Rc::clone(&client));
    let solides = Solides::new(client, now);
    let sources: [&dyn Source; 2] = [&gupy, &solides];
    record_run(store, &sources, setup, now, progress)
}

/// Runs `notify` once the ticket is released, so whoever hears it can start again,
/// even when `work` panics.
pub fn spawn_worker(
    ticket: RunTicket,
    work: impl FnOnce() + Send + 'static,
    notify: impl FnOnce() + Send + 'static,
) -> JoinHandle<()> {
    struct Finally<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Finally<F> {
        fn drop(&mut self) {
            if let Some(notify) = self.0.take() {
                notify();
            }
        }
    }
    std::thread::spawn(move || {
        // Dropped in reverse: the ticket goes first, then the notice.
        let _notice = Finally(Some(notify));
        let _ticket = ticket;
        work();
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::config::CrawlerConfig;
    use crate::crawler::job::Job;
    use crate::crawler::sources::DetailOutcome;
    use std::sync::Mutex;

    #[test]
    fn refuses_a_second_run() {
        let guard = RunGuard::default();
        let first = guard.try_start().unwrap();
        assert!(guard.is_running());
        assert_eq!(guard.try_start().err(), Some("already_running"));
        drop(first);
        assert!(!guard.is_running());
        assert!(guard.try_start().is_ok());
    }

    #[test]
    fn a_config_error_has_a_stable_code() {
        assert_eq!(config_error_code(&ConfigError::Missing), "config_missing");
        assert_eq!(
            config_error_code(&ConfigError::ProfileUnreadable),
            "profile_unreadable"
        );
        assert_eq!(
            config_error_code(&ConfigError::NoSkills),
            "profile_no_skills"
        );
        assert_eq!(
            config_error_code(&ConfigError::Invalid("min_score".into())),
            "config_invalid:min_score"
        );
        assert_eq!(
            config_error_code(&ConfigError::Unreadable("x".into())),
            "config_unreadable"
        );
    }

    #[test]
    fn the_notice_comes_after_the_ticket_even_on_a_panic() {
        let guard = RunGuard::default();
        let seen = Arc::new(Mutex::new(None));
        let (watcher, slot) = (guard.clone(), Arc::clone(&seen));
        let handle = spawn_worker(
            guard.try_start().unwrap(),
            || panic!("worker failed on purpose"),
            move || *slot.lock().unwrap() = Some(watcher.is_running()),
        );
        assert!(handle.join().is_err());
        assert_eq!(*seen.lock().unwrap(), Some(false));
    }

    struct NoJobs;
    impl Source for NoJobs {
        fn name(&self) -> &'static str {
            "gupy"
        }
        fn owns(&self, source: &str) -> bool {
            source == "gupy"
        }
        fn collect(&self, _: &dyn Fn(&Job) -> bool, _: &dyn Fn(&str)) -> Result<Vec<Job>, String> {
            Err("fora do ar".into())
        }
        fn fetch_detail(&self, _: &str) -> DetailOutcome {
            DetailOutcome::Failed("no page".into())
        }
    }

    #[test]
    fn a_run_is_recorded_with_its_stats() {
        let store = SqliteStore::open_in_memory().unwrap();
        let setup = CrawlSetup {
            config: CrawlerConfig {
                profile_path: "/data/perfil.md".into(),
                min_score: 70,
                min_skills: 3,
                filters: Default::default(),
            },
            skills: ["python".to_owned()].into_iter().collect(),
        };
        let now = crate::clock::fixed_now();
        let stats = record_run(&store, &[&NoJobs as &dyn Source], &setup, now, &|_| {}).unwrap();
        assert_eq!(stats.sources["gupy"].error.as_deref(), Some("fora do ar"));
        let run = store.last_run().unwrap().unwrap();
        assert_eq!(run.started_at, now.to_rfc3339());
        assert!(run.finished_at.is_some());
        let saved: serde_json::Value = serde_json::from_str(&run.stats).unwrap();
        assert_eq!(saved["sources"]["gupy"]["error"], "fora do ar");
    }
}
