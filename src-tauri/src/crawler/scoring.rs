//! Deterministic 0-100 fit, as `job_radar/scoring.py`. It orders jobs; it does not
//! replace reading the ad. Skills weigh more than the title, because titles lie.

use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::clock::Now;
use crate::crawler::filters::{Seniority, WorkMode, detect_seniority, work_mode_of};
use crate::crawler::job::Job;

/// Title families of the target positioning, each with its weight.
static TARGET_PATTERNS: LazyLock<Vec<(Regex, i32)>> = LazyLock::new(|| {
    [
        (
            r"(?i)\b(ai|i\.?a\.?|intelig[êe]ncia artificial|llm|genai|machine learning|ml)\b",
            30,
        ),
        (r"(?i)\b(dados|data)\b", 20),
        (
            r"(?i)\b(tech lead|l[íi]der t[ée]cnico|staff|principal|arquitet\w*)\b",
            20,
        ),
        (r"(?i)\b(full ?stack|back ?end|software)\b", 15),
    ]
    .into_iter()
    .map(|(pattern, weight)| (Regex::new(pattern).unwrap(), weight))
    .collect()
});

/// Stacks that dominate an ad and are not the user's: a weak fit.
static FOREIGN_STACK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(salesforce|sap|abap|cobol|mainframe|delphi|sharepoint|power ?bi|dynamics)\b",
    )
    .unwrap()
});

pub fn score_job(job: &Job, hits: usize, now: Now) -> u8 {
    // Skill overlap: up to 45 points, saturating at 9 hits.
    let mut score = i32::try_from(hits.min(9)).unwrap_or(9) * 5;
    // Title fit: the heaviest family only, never the sum.
    let title = caseless::default_case_fold_str(&job.title);
    score += TARGET_PATTERNS
        .iter()
        .filter(|(re, _)| re.is_match(&title).unwrap_or(false))
        .map(|(_, weight)| *weight)
        .max()
        .unwrap_or(0);
    if detect_seniority(&job.title) == Seniority::Senior {
        score += 10;
    }
    // Remote is worth more than hybrid for someone filtering by remote.
    match work_mode_of(job) {
        WorkMode::Remote => score += 15,
        WorkMode::Hybrid => score += 5,
        _ => {}
    }
    if FOREIGN_STACK
        .find_iter(&job.description)
        .filter(|m| m.is_ok())
        .count()
        >= 3
    {
        score -= 25;
    }
    // An old ad loses priority (it still passes the hard filter).
    if let Some(published) = job.published_at {
        let age = (now - published).num_seconds().div_euclid(86_400);
        if age > 35 {
            score -= 10;
        } else if age > 21 {
            score -= 5;
        }
    }
    u8::try_from(score.clamp(0, 100)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::filters::count_skill_hits;
    use crate::crawler::job::Job;
    use serde_json::Value;
    use std::collections::BTreeSet;

    #[test]
    fn matches_the_python_vectors() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/parity/scoring.json")).unwrap();
        let skills: BTreeSet<String> = v["skills"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
        let rows = v["jobs"].as_array().unwrap();
        assert!(rows.len() > 50);
        for row in rows {
            let mut job = Job::new(
                "gupy",
                "1",
                row["title"].as_str().unwrap(),
                "Acme",
                "https://a.example/1",
            )
            .description(row["description"].as_str().unwrap())
            .location(row["location"].as_str().unwrap())
            .work_mode_hint(row["hint"].as_str());
            job.published_at = row["published_at"]
                .as_str()
                .map(|d| chrono::DateTime::parse_from_rfc3339(d).unwrap());
            let hits = count_skill_hits(&job, &skills);
            assert_eq!(hits.len() as u64, row["hits"].as_u64().unwrap(), "{row}");
            assert_eq!(
                u64::from(score_job(&job, hits.len(), crate::clock::fixed_now())),
                row["score"].as_u64().unwrap(),
                "{row}"
            );
        }
    }
}
