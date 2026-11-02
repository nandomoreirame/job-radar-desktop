//! Hard filters: drop a job before any scoring, as `job_radar/filters.py` does.
//! Order matters: cheap and configurable cuts first, the skill match last.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use fancy_regex::Regex;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::clock::Now;
use crate::crawler::identity::company_key;
use crate::crawler::job::Job;

/// Sources that declare it structurally (Solides `affirmative`, `pcdOnly`) tag the job.
pub const AFFIRMATIVE_TAG: &str = "vaga afirmativa";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seniority {
    Senior,
    Below,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkMode {
    Remote,
    Hybrid,
    Onsite,
    Unknown,
}

/// The user's own cuts. Neutral defaults: a filter nobody set discards nothing.
#[derive(Debug, Clone, Default)]
pub struct FilterSettings {
    pub min_salary_pj: u32,
    pub min_salary_clt: u32,
    pub commute_cities: BTreeSet<String>,
    pub max_job_age_days: u32,
    pub blocked_companies: BTreeSet<String>,
}

macro_rules! pattern {
    ($name:ident, $re:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($re).unwrap());
    };
}

pattern!(
    SENIOR,
    r"(?i)\b(s[êe]nior|senior|sr\.?\b|staff|principal|especialista|lead|l[íi]der|head|manager|gerente|arquitet\w*)"
);
// "PL" without a dot is "pleno", except in "PL/SQL", Oracle's language.
pattern!(
    BELOW,
    r"(?i)\b(j[úu]nior|junior|jr\.?\b|est[áa]gi\w*|trainee|aprendiz|plenos?\b|pl\.?\b(?!/?sql))"
);
pattern!(
    REMOTE,
    r"(?i)\b(remot[oa]|remote|home\s*office|anywhere|teletrabalho)\b"
);
pattern!(HYBRID, r"(?i)\b(h[íi]brid[oa]|hybrid)\b");
pattern!(ONSITE, r"(?i)\b(presencial|on-?site|no local)\b");
pattern!(PJ, r"(?i)\b(pj|pessoa\s+jur[íi]dica|cnpj)\b");
pattern!(
    CLT,
    r"(?i)\b(clt|carteira\s+assinada|efetiv[oa]|regime\s+celetista)\b"
);
// Needs a currency mark or a "k" suffix, so a year or a head count is never a salary.
pattern!(
    MONEY,
    r"(?i)(?:r\$\s*)(\d{1,3}(?:[.\s]\d{3})+|\d{4,6})(?:,\d{2})?|(\d{1,3})\s*k\b"
);
pattern!(
    BENEFIT,
    r"(?i)\b(vr|va|vale[\s-]?(refei[çc][ãa]o|alimenta[çc][ãa]o|transporte)|aux[íi]lio|plano de sa[úu]de|gympass)\b"
);
// A talent pool is a resume form, not an open position.
pattern!(
    NOT_A_JOB,
    r"(?i)\b(banco de talentos|cadastro de curr[íi]culo|talent pool|candidatura espont[âa]nea)\b"
);
// Affirmative jobs: the reliable signal is the noun right next to the marker.
pattern!(
    AFFIRMATIVE_TITLE,
    r"(?i)(?:exclusiv|afirmativ|destinad|reservad)\w*\W+(?:\w+\W+){0,3}?(?:mulher(?:es)?|negr[ao]s?|pretos?|pardos?|pcd|pessoas?\s+com\s+defici[êe]ncia|lgbtq?i?a?\+?|ind[íi]genas?|trans|50\+|refugiad[ao]s?)"
);
pattern!(
    AFFIRMATIVE_DESC,
    r"(?i)\b(?:vagas?|posi[çc][ãa]o|posi[çc][õo]es|oportunidades?|processo\s+seletivo)\s+(?:é|s[ãa]o|est[áa]\s+)?(?:exclusiv|afirmativ|destinad|reservad)\w*\s+(?:a|ao|à|aos|às|para)\s+(?:\w+\s+){0,3}?(?:mulher(?:es)?|negr[ao]s?|pretos?|pardos?|pcd|pessoas?\s+com\s+defici[êe]ncia|lgbtq?i?a?\+?|ind[íi]genas?|trans|50\+|refugiad[ao]s?)"
);
// `java` must not match inside "javascript": the trailing \b guarantees it.
pattern!(
    EXCLUDED_LANG,
    r"(?i)(\bjava\b|\bkotlin\b|\bc#|\bc\+\+|\.net\b|\bdotnet\b|\basp\.net\b|\bspring\s*boot\b|\bdelphi\b|\bcobol\b|\babap\b|\bsap\b|\bvisual\s*basic\b|\bvb\.net\b|\bscala\b|\bperl\b)"
);
pattern!(
    PROFILE_LANG,
    r"(?i)(\bpython\b|\btypescript\b|\bjavascript\b|\bnode(\.?js)?\b|\breact\b|\bnext(\.?js)?\b|\brust\b|\bgolang\b|\bgo\b|\bphp\b|\blaravel\b|\bfastapi\b|\bdjango\b|\bpolars\b|\bpandas\b)"
);
// The role lives in the TITLE only: "the QA team" is routine in a developer job.
pattern!(
    EXCLUDED_ROLE,
    r"(?i)(\bq\.?a\.?\b|\bsdet\b|\btesters?\b|quality\s+assurance|test\s+engineer|(analista|engenheir[oa]|especialista|l[íi]der)\s+(de\s+)?(testes?|qualidade\s+de\s+software)|automa[çc][ãa]o\s+de\s+testes|garantia\s+de\s+qualidade|scrum\s+master|product\s+owner|agile\s+(coach|master)|\bdba\b|administrador\s+de\s+banco|suporte\s+t[ée]cnico|service\s+desk|help\s+desk|ux/ui|(ux|ui|product)\s+designer|recrutador|business\s+analyst)"
);

fn hit(re: &Regex, text: &str) -> bool {
    re.is_match(text).unwrap_or(false)
}

fn strip_accents(text: &str) -> String {
    text.nfd().filter(|c| !is_combining_mark(*c)).collect()
}

fn fold_city(text: &str) -> String {
    caseless::default_case_fold_str(strip_accents(text).trim())
}

/// A hybrid job is viable only in a configured city; no cities means no limit.
pub fn in_commute_range(location: &str, cities: &BTreeSet<String>) -> bool {
    if cities.is_empty() {
        return true;
    }
    if location.is_empty() {
        return false;
    }
    let wanted: BTreeSet<String> = cities
        .iter()
        .map(|c| fold_city(c))
        .filter(|c| !c.is_empty())
        .collect();
    if wanted.is_empty() {
        return true;
    }
    wanted.contains(&fold_city(location.split(',').next().unwrap_or("")))
}

/// Lowest plausible monthly amount: "up to R$ 20.000" does not promise 20.000.
pub fn parse_salary(text: &str) -> Option<u32> {
    let chars: Vec<char> = text.chars().collect();
    let char_at = |byte: usize| text[..byte].chars().count();
    let mut values = Vec::new();
    for found in MONEY.captures_iter(text) {
        let Ok(caps) = found else { continue };
        let Some(whole) = caps.get(0) else { continue };
        let (start, end) = (char_at(whole.start()), char_at(whole.end()));
        let window: String = chars[start.saturating_sub(40)..(end + 20).min(chars.len())]
            .iter()
            .collect();
        if hit(&BENEFIT, &window) {
            continue;
        }
        if let Some(group) = caps.get(1) {
            let digits: String = group
                .as_str()
                .chars()
                .filter(|c| *c != '.' && !c.is_whitespace())
                .collect();
            values.extend(digits.parse::<u32>().ok());
        } else if let Some(group) = caps.get(2) {
            values.extend(group.as_str().parse::<u32>().ok().map(|v| v * 1000));
        }
    }
    values.into_iter().min()
}

pub fn detect_contract(text: &str) -> Option<&'static str> {
    if hit(&PJ, text) {
        Some("PJ")
    } else if hit(&CLT, text) {
        Some("CLT")
    } else {
        None
    }
}

/// From the title. "Pleno/Sênior" counts as senior: the door is open.
pub fn detect_seniority(title: &str) -> Seniority {
    if title.is_empty() {
        Seniority::Unknown
    } else if hit(&SENIOR, title) {
        Seniority::Senior
    } else if hit(&BELOW, title) {
        Seniority::Below
    } else {
        Seniority::Unknown
    }
}

/// Remote beats hybrid, which beats on-site: ads tend to name the most permissive.
pub fn detect_work_mode(text: &str) -> WorkMode {
    if hit(&REMOTE, text) {
        WorkMode::Remote
    } else if hit(&HYBRID, text) {
        WorkMode::Hybrid
    } else if hit(&ONSITE, text) {
        WorkMode::Onsite
    } else {
        WorkMode::Unknown
    }
}

/// A mode the source declares beats any text heuristic.
pub fn mode_from_hint(hint: Option<&str>) -> Option<WorkMode> {
    let normalized = caseless::default_case_fold_str(hint?.trim()).replace(['-', '_'], "");
    match normalized.as_str() {
        "remote" => Some(WorkMode::Remote),
        "hybrid" => Some(WorkMode::Hybrid),
        "onsite" | "presencial" => Some(WorkMode::Onsite),
        _ => None,
    }
}

pub fn work_mode_of(job: &Job) -> WorkMode {
    mode_from_hint(job.work_mode_hint.as_deref())
        .unwrap_or_else(|| detect_work_mode(&job.haystack()))
}

const BOUNDARY: &str = "abcdefghijklmnopqrstuvwxyz0123456789+#";

/// Profile skills found in the job, matched whole: "c#" never inside "c#x".
pub fn count_skill_hits(job: &Job, skills: &BTreeSet<String>) -> BTreeSet<String> {
    let hay = job.haystack();
    skills
        .iter()
        .filter(|skill| {
            let needle = caseless::default_case_fold_str(skill);
            if needle.is_empty() {
                return false;
            }
            let mut from = 0;
            while let Some(at) = hay[from..].find(&needle) {
                let start = from + at;
                let end = start + needle.len();
                let before_ok = hay[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !BOUNDARY.contains(c) && c != '.');
                let after_ok = hay[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !BOUNDARY.contains(c));
                if before_ok && after_ok {
                    return true;
                }
                from = start + hay[start..].chars().next().map_or(1, char::len_utf8);
            }
            false
        })
        .cloned()
        .collect()
}

pub fn is_affirmative(job: &Job) -> bool {
    job.tags
        .iter()
        .any(|t| caseless::default_case_fold_str(t) == AFFIRMATIVE_TAG)
        || hit(&AFFIRMATIVE_TITLE, &job.title)
        || hit(&AFFIRMATIVE_DESC, &job.description)
}

pub fn has_excluded_role(job: &Job) -> bool {
    hit(&EXCLUDED_ROLE, &job.title)
}

fn terms(re: &Regex, text: &str) -> BTreeSet<String> {
    re.find_iter(text)
        .filter_map(Result::ok)
        .map(|m| caseless::default_case_fold_str(m.as_str()))
        .collect()
}

/// Decides by weight, not presence: Java listed as one option in a Python ad is not the job.
pub fn has_excluded_language(job: &Job) -> bool {
    let foreign_title = terms(&EXCLUDED_LANG, &job.title);
    if !foreign_title.is_empty() && terms(&PROFILE_LANG, &job.title).is_empty() {
        return true;
    }
    let text = format!("{} {}", job.title, job.description);
    let foreign = terms(&EXCLUDED_LANG, &text);
    !foreign.is_empty() && foreign.len() > terms(&PROFILE_LANG, &text).len()
}

/// Thousands with dots, as Python's `{:,}` followed by `.replace(",", ".")`.
fn thousands(value: u32) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// The non-negotiable cuts. `Err` carries the same pt-BR reason as Python.
pub fn passes_hard_filters(
    job: &Job,
    skills: &BTreeSet<String>,
    min_skill_hits: usize,
    settings: &FilterSettings,
    now: Now,
) -> Result<(), String> {
    if settings.max_job_age_days > 0
        && let Some(published) = job.published_at
    {
        let age = (now - published).num_seconds().div_euclid(86_400).max(0);
        if age > i64::from(settings.max_job_age_days) {
            return Err(format!("vaga antiga ({age} dias)"));
        }
    }
    if !settings.blocked_companies.is_empty() {
        let blocked: BTreeSet<String> = settings
            .blocked_companies
            .iter()
            .map(|c| company_key(c))
            .filter(|c| !c.is_empty())
            .collect();
        if blocked.contains(&company_key(&job.company)) {
            return Err("empresa bloqueada".into());
        }
    }
    if hit(&NOT_A_JOB, &job.title) {
        return Err("não é vaga (banco de talentos)".into());
    }
    if detect_seniority(&job.title) == Seniority::Below {
        return Err("senioridade abaixo de sênior".into());
    }
    if is_affirmative(job) {
        return Err("vaga afirmativa (reservada a grupo específico)".into());
    }
    if has_excluded_role(job) {
        return Err("função fora do perfil".into());
    }
    if has_excluded_language(job) {
        return Err("linguagem principal fora do perfil".into());
    }
    match work_mode_of(job) {
        WorkMode::Onsite => return Err("modalidade presencial".into()),
        WorkMode::Hybrid if !in_commute_range(&job.location, &settings.commute_cities) => {
            return Err("modalidade híbrida fora da região configurada".into());
        }
        _ => {}
    }
    if let Some(salary) = parse_salary(&job.description) {
        let floor = if detect_contract(&job.haystack()) == Some("PJ") {
            settings.min_salary_pj
        } else {
            settings.min_salary_clt
        };
        if floor > 0 && salary < floor {
            return Err(format!(
                "salário declarado (R$ {}) abaixo do piso de R$ {}",
                thousands(salary),
                thousands(floor)
            ));
        }
    }
    let hits = count_skill_hits(job, skills).len();
    if hits < min_skill_hits {
        return Err(format!(
            "qualificações insuficientes ({hits} de {min_skill_hits} exigidas)"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::job::Job;
    use serde_json::Value;
    use std::collections::BTreeSet;

    fn vectors() -> Value {
        serde_json::from_str(include_str!("../../tests/fixtures/parity/filters.json")).unwrap()
    }

    fn settings() -> FilterSettings {
        FilterSettings {
            min_salary_pj: 10_000,
            min_salary_clt: 9_000,
            commute_cities: ["São Paulo".to_owned()].into_iter().collect(),
            max_job_age_days: 0,
            blocked_companies: ["Bloqueada S.A.".to_owned()].into_iter().collect(),
        }
    }

    #[test]
    fn matches_the_python_vectors() {
        let v = vectors();
        let skills: BTreeSet<String> = v["skills"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
        for row in v["jobs"].as_array().unwrap() {
            let job = Job::new(
                "gupy",
                "1",
                row["title"].as_str().unwrap(),
                "Acme",
                "https://a.example/1",
            )
            .description(row["description"].as_str().unwrap())
            .location(row["location"].as_str().unwrap())
            .work_mode_hint(row["hint"].as_str());
            let verdict =
                passes_hard_filters(&job, &skills, 3, &settings(), crate::clock::fixed_now());
            let ok = verdict.is_ok();
            let reason = verdict.err().unwrap_or_default();
            assert_eq!(ok, row["ok"].as_bool().unwrap(), "{row}");
            assert_eq!(reason, row["reason"].as_str().unwrap(), "{row}");
            let hits: Vec<String> = count_skill_hits(&job, &skills).into_iter().collect();
            let expected: Vec<String> = row["hits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_owned())
                .collect();
            assert_eq!(hits, expected, "{row}");
        }
        for row in v["salaries"].as_array().unwrap() {
            assert_eq!(
                parse_salary(row[0].as_str().unwrap()),
                row[1].as_u64().map(|n| n as u32),
                "{row}"
            );
        }
        for row in v["contracts"].as_array().unwrap() {
            assert_eq!(
                detect_contract(row[0].as_str().unwrap()),
                row[1].as_str(),
                "{row}"
            );
        }
        assert!(
            v["jobs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| !r["ok"].as_bool().unwrap())
        );
        assert!(
            v["jobs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["ok"].as_bool().unwrap())
        );
    }

    #[test]
    fn blocks_a_company_by_key() {
        let skills: BTreeSet<String> = ["python", "aws", "docker"]
            .map(str::to_owned)
            .into_iter()
            .collect();
        let job = Job::new(
            "gupy",
            "1",
            "Dev Sênior",
            "BLOQUEADA",
            "https://a.example/1",
        )
        .description("Python AWS Docker")
        .work_mode_hint(Some("remote"));
        assert_eq!(
            passes_hard_filters(&job, &skills, 3, &settings(), crate::clock::fixed_now()),
            Err(vectors()["blocked"][1].as_str().unwrap().to_owned())
        );
    }
}
