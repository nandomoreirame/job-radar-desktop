use crate::domain::Target;
use caseless::default_case_fold_str;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::LazyLock;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
pub fn fold(text: &str) -> String {
    let plain: String = text.nfkd().filter(|c| !is_combining_mark(*c)).collect();
    let lowered = default_case_fold_str(&plain);
    lowered
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c.is_ascii_lowercase() {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn strip(title: &str) -> String {
    static SEP: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)^(?P<head>.*?)\s*[-|/]\s*(?P<tail>[^-|/]+)$").unwrap());
    let mut text = title.trim().to_owned();
    while let Some(m) = SEP.captures(&text) {
        let tail = fold(m.name("tail").unwrap().as_str());
        if [
            "remoto",
            "remote",
            "hibrido",
            "hybrid",
            "presencial",
            "onsite",
            "sao paulo",
            "rio de janeiro",
            "brasil",
        ]
        .contains(&tail.as_str())
        {
            text = m.name("head").unwrap().as_str().trim().into()
        } else {
            break;
        }
    }
    text
}
pub fn title_key(title: &str) -> String {
    let mut words = fold(&strip(title))
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    words.sort();
    words.dedup();
    words.join(" ")
}
pub fn company_key(company: &str) -> String {
    let mut v = fold(company);
    for prefix in ["grupo "] {
        if v.starts_with(prefix) {
            v = v[prefix.len()..].into()
        }
    }
    for suffix in [
        " ltda me", " ltda", " s a", " sa", " eireli", " inc", " llc", " ltd", " corp", " gmbh",
        " me",
    ] {
        if v.ends_with(suffix) && v.len() > suffix.len() {
            v.truncate(v.len() - suffix.len());
            break;
        }
    }
    v.trim().into()
}
pub fn job_fingerprint(company: &str, title: &str) -> String {
    let c = company_key(company);
    let joined = if c.is_empty() {
        title_key(title)
    } else {
        format!("{c}|{}", title_key(title))
    };
    Sha256::digest(joined.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn target_from_title(title: &str) -> Option<Target> {
    let t = fold(title);
    if t.split_whitespace()
        .any(|x| ["ai", "ia", "llm", "genai", "machine", "ml"].contains(&x))
    {
        Some(Target::AiLlm)
    } else if t.split_whitespace().any(|x| ["data", "dados"].contains(&x)) {
        Some(Target::Data)
    } else if t.contains("tech lead") || t.contains("staff") || t.contains("principal") {
        Some(Target::TechLead)
    } else if t.contains("full stack") || t.contains("backend") || t.contains("software") {
        Some(Target::FullStack)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_basics() {
        assert_eq!(title_key("Backend Sr"), title_key("Sr Backend"));
        assert_eq!(job_fingerprint("Acme", "X").len(), 16);
    }
}
