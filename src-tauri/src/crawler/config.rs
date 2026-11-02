//! Crawler settings. The file lives in the app data directory because an app opened
//! from a launcher gets no environment; the settings page will write this same file.
//! Defaults are neutral: a filter nobody configured discards nothing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::crawler::filters::FilterSettings;
use crate::crawler::profile::load_profile;

pub const CONFIG_FILE: &str = "crawler.json";

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConfigError {
    Missing,
    Unreadable(String),
    Invalid(String),
    ProfileUnreadable,
    NoSkills,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    profile_path: PathBuf,
    #[serde(default = "default_min_score")]
    min_score: u32,
    #[serde(default = "default_min_skills")]
    min_skills: u32,
    #[serde(default)]
    min_salary_pj: u32,
    #[serde(default)]
    min_salary_clt: u32,
    #[serde(default)]
    commute_cities: BTreeSet<String>,
    #[serde(default)]
    max_job_age_days: u32,
    #[serde(default)]
    blocked_companies: BTreeSet<String>,
}
fn default_min_score() -> u32 {
    70
}
fn default_min_skills() -> u32 {
    3
}

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub profile_path: PathBuf,
    pub min_score: u8,
    pub min_skills: usize,
    pub filters: FilterSettings,
}

#[derive(Debug, Clone)]
pub struct CrawlSetup {
    pub config: CrawlerConfig,
    pub skills: BTreeSet<String>,
}

pub fn load_config(path: &Path) -> Result<CrawlerConfig, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ConfigError::Missing);
        }
        Err(error) => return Err(ConfigError::Unreadable(error.to_string())),
    };
    let raw: RawConfig =
        serde_json::from_str(&text).map_err(|e| ConfigError::Unreadable(e.to_string()))?;
    let min_score = u8::try_from(raw.min_score)
        .ok()
        .filter(|v| *v <= 100)
        .ok_or_else(|| ConfigError::Invalid("min_score".into()))?;
    Ok(CrawlerConfig {
        profile_path: raw.profile_path,
        min_score,
        min_skills: raw.min_skills as usize,
        filters: FilterSettings {
            min_salary_pj: raw.min_salary_pj,
            min_salary_clt: raw.min_salary_clt,
            commute_cities: raw.commute_cities,
            max_job_age_days: raw.max_job_age_days,
            blocked_companies: raw.blocked_companies,
        },
    })
}

pub fn load_crawl_setup(path: &Path) -> Result<CrawlSetup, ConfigError> {
    let config = load_config(path)?;
    let skills = load_profile(&config.profile_path).map_err(|_| ConfigError::ProfileUnreadable)?;
    if skills.is_empty() {
        return Err(ConfigError::NoSkills);
    }
    Ok(CrawlSetup { config, skills })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &std::path::Path, name: &str, text: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        path
    }

    const PROFILE: &str = "## Stack técnica\n\n| **Linguagens** | Python, Rust |\n";

    #[test]
    fn reads_defaults_and_names_the_invalid_field() {
        let dir = tempfile::tempdir().unwrap();
        let profile = write(dir.path(), "cv.md", PROFILE);
        let path = write(
            dir.path(),
            "crawler.json",
            &serde_json::json!({ "profile_path": profile }).to_string(),
        );
        let config = load_config(&path).unwrap();
        assert_eq!((config.min_score, config.min_skills), (70, 3));
        assert_eq!(config.filters.min_salary_pj, 0);
        assert!(
            config.filters.commute_cities.is_empty() && config.filters.blocked_companies.is_empty()
        );

        let bad = write(
            dir.path(),
            "bad.json",
            &serde_json::json!({ "profile_path": profile, "min_score": 500 }).to_string(),
        );
        assert_eq!(
            load_config(&bad).unwrap_err(),
            ConfigError::Invalid("min_score".into())
        );
        let not_json = write(dir.path(), "x.json", "{ nope");
        assert!(matches!(
            load_config(&not_json),
            Err(ConfigError::Unreadable(_))
        ));
    }

    #[test]
    fn refuses_a_missing_or_empty_profile() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load_config(&dir.path().join("none.json")).unwrap_err(),
            ConfigError::Missing
        );
        let missing = write(
            dir.path(),
            "a.json",
            &serde_json::json!({ "profile_path": dir.path().join("none.md") }).to_string(),
        );
        assert!(matches!(
            load_crawl_setup(&missing),
            Err(ConfigError::ProfileUnreadable)
        ));
        let empty_cv = write(dir.path(), "cv.md", "# CV sem a seção\n");
        let empty = write(
            dir.path(),
            "b.json",
            &serde_json::json!({ "profile_path": empty_cv }).to_string(),
        );
        assert_eq!(load_crawl_setup(&empty).unwrap_err(), ConfigError::NoSkills);
        let good_cv = write(dir.path(), "ok.md", PROFILE);
        let good = write(
            dir.path(),
            "c.json",
            &serde_json::json!({ "profile_path": good_cv }).to_string(),
        );
        assert!(load_crawl_setup(&good).unwrap().skills.contains("python"));
    }
}
