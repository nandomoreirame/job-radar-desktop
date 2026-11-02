//! Job identity, byte-identical to `job_radar/identity.py`: imported fingerprints
//! decide whether a crawled job is already known. Errs on the side of NOT merging:
//! over-merging hides a job, under-merging only shows a duplicate.

use std::sync::LazyLock;

use caseless::default_case_fold_str;
use regex::Regex;
use sha2::{Digest, Sha256};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Legal suffixes, already folded; longer first so "ltda me" goes before "ltda"/"me".
const COMPANY_SUFFIXES: [&str; 12] = [
    "ltda me", "ltda", "s a", "sa", "eireli", "epp", "inc", "llc", "ltd", "corp", "gmbh", "me",
];

/// Brazilian state codes. Alone they are NOT a tag: "GO"/"TO" collide with the Go
/// language and real title words.
const UF: [&str; 27] = [
    "ac", "al", "ap", "am", "ba", "ce", "df", "es", "go", "ma", "mt", "ms", "mg", "pa", "pb", "pr",
    "pe", "pi", "rj", "rn", "rs", "ro", "rr", "sc", "sp", "se", "to",
];

const MODALITY: [&str; 16] = [
    "remoto",
    "remota",
    "remote",
    "hibrido",
    "hibrida",
    "hybrid",
    "presencial",
    "onsite",
    "on",
    "site",
    "home",
    "office",
    "homeoffice",
    "teletrabalho",
    "anywhere",
    "100",
];

/// City phrases, matched whole so "porto"/"rio"/"go" alone stay in a real title.
const CITY_PHRASES: [&str; 35] = [
    "sao paulo",
    "rio de janeiro",
    "rio janeiro",
    "belo horizonte",
    "porto alegre",
    "campo grande",
    "ribeirao preto",
    "brasilia",
    "curitiba",
    "salvador",
    "recife",
    "fortaleza",
    "manaus",
    "belem",
    "goiania",
    "campinas",
    "guarulhos",
    "osasco",
    "barueri",
    "santos",
    "niteroi",
    "florianopolis",
    "vitoria",
    "cuiaba",
    "joinville",
    "sorocaba",
    "uberlandia",
    "londrina",
    "maringa",
    "cotia",
    "jandira",
    "alphaville",
    "brasil",
    "brazil",
    "latam",
];

static TRAILING_SEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<head>.*?)\s*[-|/]\s*(?P<tail>[^-|/]+)$").unwrap());
static TRAILING_PAREN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<head>.*?)\s*\((?P<inner>[^()]*)\)\s*$").unwrap());

/// NFKD without marks, case folded; anything outside `[0-9a-z]` becomes one space.
pub fn fold(text: &str) -> String {
    let plain: String = text.nfkd().filter(|c| !is_combining_mark(*c)).collect();
    let lowered = default_case_fold_str(&plain);
    let spaced: String = lowered
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c.is_ascii_lowercase() {
                c
            } else {
                ' '
            }
        })
        .collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn folded_endswith_city(head: &str) -> bool {
    !head.is_empty()
        && CITY_PHRASES
            .iter()
            .any(|phrase| head == *phrase || head.ends_with(&format!(" {phrase}")))
}

/// True when the segment is only work mode or city (a state code only after a city).
fn segment_is_location_tag(segment: &str, head: &str) -> bool {
    let folded = fold(segment);
    let tokens: Vec<&str> = folded.split_whitespace().collect();
    if tokens.is_empty() || tokens.iter().all(|t| MODALITY.contains(t)) {
        return true;
    }
    let body: Vec<&str> = tokens
        .iter()
        .copied()
        .skip_while(|t| MODALITY.contains(t))
        .collect();
    if body.is_empty() {
        return true;
    }
    if CITY_PHRASES.contains(&body.join(" ").as_str()) {
        return true;
    }
    if body.len() == 1 && UF.contains(&body[0]) {
        return folded_endswith_city(&fold(head));
    }
    if body.len() >= 2 && UF.contains(&body[body.len() - 1]) {
        return CITY_PHRASES.contains(&body[..body.len() - 1].join(" ").as_str());
    }
    false
}

/// Drops location and work-mode suffixes that sources glue to the title.
fn strip_trailing_tags(title: &str) -> String {
    let mut text = title.trim().to_owned();
    while !text.is_empty() {
        if let Some(m) = TRAILING_PAREN.captures(&text)
            && segment_is_location_tag(&m["inner"], &m["head"])
        {
            text = m["head"].trim_end().to_owned();
            continue;
        }
        if let Some(m) = TRAILING_SEP.captures(&text)
            && segment_is_location_tag(&m["tail"], &m["head"])
        {
            text = m["head"].trim_end().to_owned();
            continue;
        }
        break;
    }
    text
}

/// Words of the cleaned title as a sorted set: "Backend Sr" equals "Sr Backend".
pub fn title_key(title: &str) -> String {
    let folded = fold(&strip_trailing_tags(title));
    let words: std::collections::BTreeSet<&str> = folded.split_whitespace().collect();
    words.into_iter().collect::<Vec<_>>().join(" ")
}

/// Company key without legal suffix nor a leading "Grupo ".
pub fn company_key(company: &str) -> String {
    let original = fold(company);
    if original.is_empty() {
        return original;
    }
    let mut folded = original.clone();
    loop {
        let mut next = folded.clone();
        if next.starts_with("grupo ") && next.len() > 6 {
            next = next[6..].to_owned();
        }
        for suffix in COMPANY_SUFFIXES {
            let token = format!(" {suffix}");
            if next.ends_with(&token) {
                let candidate = next[..next.len() - token.len()].trim_end();
                if !candidate.is_empty() {
                    next = candidate.to_owned();
                    break;
                }
            }
            if next == suffix {
                // Stripping would empty it: keep the value from before this pass.
                next = folded.clone();
                break;
            }
        }
        if next == folded {
            break;
        }
        folded = next;
    }
    if folded.is_empty() { original } else { folded }
}

/// `sha256(company_key|title_key)[:16]`; title only when the company is empty.
pub fn job_fingerprint(company: &str, title: &str) -> String {
    let tkey = title_key(title);
    let ckey = company_key(company);
    let joined = if ckey.is_empty() {
        tkey
    } else {
        format!("{ckey}|{tkey}")
    };
    Sha256::digest(joined.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_vectors() {
        let vectors: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/parity/identity.json"))
                .unwrap();
        let rows = |key: &str| vectors[key].as_array().unwrap().clone();
        let text = |value: &Value| value.as_str().unwrap().to_owned();
        for row in rows("titles") {
            assert_eq!(title_key(&text(&row[0])), text(&row[1]), "title {row}");
        }
        for row in rows("companies") {
            assert_eq!(company_key(&text(&row[0])), text(&row[1]), "company {row}");
        }
        for row in rows("fingerprints") {
            assert_eq!(
                job_fingerprint(&text(&row[0]), &text(&row[1])),
                text(&row[2]),
                "fingerprint {row}"
            );
        }
        for row in rows("urls") {
            assert_eq!(
                crate::identity::url_key(&text(&row[0])),
                text(&row[1]),
                "url {row}"
            );
        }
        assert!(
            rows("fingerprints").len() > 100,
            "the vectors must not be empty"
        );
    }
}
