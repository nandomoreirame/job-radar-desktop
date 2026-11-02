//! Skills come from the resume's "Stack técnica" table, as `job_radar/profile.py`:
//! the resume is kept current, a second list would age in silence.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

/// Resume words that are not a searchable skill in an ad.
const STOPWORDS: [&str; 15] = [
    "e",
    "de",
    "da",
    "do",
    "em",
    "com",
    "para",
    "produção",
    "app router",
    "design systems",
    "arquitetura hexagonal",
    "acessibilidade",
    "testes e2e",
    "engenharia",
    "agentes",
];

/// The resume and the ad name one skill differently. Expansion always starts from a
/// term the resume DECLARES: nobody gets "llm" for knowing "python".
const SYNONYMS: [(&str, &[&str]); 17] = [
    (
        "harness de agentes",
        &[
            "llm",
            "llms",
            "agentes de ia",
            "ai agents",
            "ia generativa",
            "genai",
            "inteligência artificial",
            "agentic",
        ],
    ),
    (
        "orquestração multi-agente",
        &[
            "multi-agente",
            "multiagente",
            "orquestração de agentes",
            "agentes de ia",
            "llm",
            "llms",
            "ia generativa",
        ],
    ),
    ("subagentes", &["agentes de ia", "ai agents"]),
    ("claude code", &["claude", "anthropic"]),
    ("codex", &["openai"]),
    ("gemini", &["vertex ai"]),
    ("mcp", &["model context protocol"]),
    ("next.js", &["nextjs", "next js"]),
    ("node.js", &["nodejs", "node js"]),
    ("ci/cd", &["cicd", "integração contínua"]),
    ("gcp", &["google cloud"]),
    ("aws", &["amazon web services"]),
    (
        "etl em produção",
        &["etl", "pipeline de dados", "data pipeline"],
    ),
    (
        "normalização e validação de dados",
        &["qualidade de dados", "data quality"],
    ),
    ("tdd", &["test driven development"]),
    ("testes e2e com playwright", &["playwright", "testes e2e"]),
    ("vue.js e nuxt", &["vue", "nuxt"]),
];

static SECTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?im)^##\s+Stack\s+t[ée]cnica\s*$").unwrap());
static ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\|\s*\*\*(?P<area>[^*]+)\*\*\s*\|(?P<items>.+?)\|\s*$").unwrap()
});
static PAREN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([^)]*)\)").unwrap());
static SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[,;]").unwrap());
static VERSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+\d+(\.\d+)?\+?$").unwrap());

fn fold(text: &str) -> String {
    caseless::default_case_fold_str(text)
}

/// Adds the spellings ads use, keeping the resume's own.
pub fn expand_synonyms(skills: BTreeSet<String>) -> BTreeSet<String> {
    let mut out = skills.clone();
    for term in &skills {
        if let Some((_, extra)) = SYNONYMS.iter().find(|(key, _)| *key == fold(term)) {
            out.extend(extra.iter().map(|s| (*s).to_owned()));
        }
    }
    out
}

/// Splits a table cell into terms, opening what is inside parentheses.
fn split_terms(cell: &str) -> Vec<String> {
    let expanded = PAREN.replace_all(cell, ", $1");
    SPLIT
        .split(&expanded)
        .filter_map(|raw| {
            let term = raw
                .trim()
                .trim_matches(['*', '`', ' '])
                .trim_end_matches('.');
            let term = VERSION.replace(term, "").into_owned();
            (term.chars().count() >= 2 && !STOPWORDS.contains(&fold(&term).as_str()))
                .then_some(term)
        })
        .collect()
}

pub fn extract_skills(markdown: &str) -> BTreeSet<String> {
    let Some(found) = SECTION.find(markdown) else {
        return BTreeSet::new();
    };
    let tail = &markdown[found.end()..];
    let block = tail.find("\n## ").map_or(tail, |end| &tail[..end]);
    ROW.captures_iter(block)
        .flat_map(|caps| split_terms(&caps["items"]))
        .map(|term| fold(&term))
        .collect()
}

/// The profile's skills, with synonyms.
pub fn load_profile(path: &Path) -> std::io::Result<BTreeSet<String>> {
    Ok(expand_synonyms(extract_skills(&std::fs::read_to_string(
        path,
    )?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_skills() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/parity/scoring.json")).unwrap();
        let list = |key: &str| -> BTreeSet<String> {
            v[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_owned())
                .collect()
        };
        let profile = include_str!("../../tests/fixtures/perfil.md");
        assert_eq!(extract_skills(profile), list("extracted"));
        assert_eq!(expand_synonyms(extract_skills(profile)), list("skills"));
        assert!(!list("skills").is_empty());
    }

    #[test]
    fn a_profile_without_the_section_has_no_skills() {
        assert!(extract_skills("# CV\n\n## Experiência\n").is_empty());
    }
}
