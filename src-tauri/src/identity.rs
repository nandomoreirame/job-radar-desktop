use crate::domain::Target;
use regex::Regex;
use std::sync::LazyLock;
use url::Url;
const TRACKING: [&str; 2] = ["gclid", "fbclid"];
const HOST_TRACKING: [(&str, &[&str]); 4] = [
    (
        "linkedin.com",
        &[
            "trk",
            "trackingid",
            "refid",
            "ebp",
            "lipi",
            "originalsubdomain",
        ],
    ),
    ("greenhouse.io", &["gh_src"]),
    ("lever.co", &["lever-source", "lever-origin"]),
    ("ashbyhq.com", &["ashby_src"]),
];
pub fn url_key(raw: &str) -> String {
    let Ok(parsed) = Url::parse(raw.trim()) else {
        return String::new();
    };
    // Any scheme with a host maps to https, as in Python: this is a key, never a link.
    let Some(host) = parsed.host_str() else {
        return String::new();
    };
    let host = host.to_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_owned();
    let netloc = match parsed.port() {
        Some(port) if port != 80 && port != 443 => format!("{host}:{port}"),
        _ => host.clone(),
    };
    // Python's urlparse keeps "/" only when the URL wrote it; `url` always adds it.
    let wrote_root = raw
        .trim()
        .split_once("://")
        .and_then(|(_, rest)| {
            rest.find(['/', '?', '#'])
                .map(|at| rest[at..].starts_with('/'))
        })
        .unwrap_or(false);
    let path = if parsed.path() == "/" {
        if wrote_root { "/" } else { "" }
    } else {
        parsed.path().trim_end_matches('/')
    };
    let host_params = HOST_TRACKING
        .iter()
        .find(|(suffix, _)| host == *suffix || host.ends_with(&format!(".{suffix}")))
        .map_or(&[][..], |(_, params)| *params);
    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(key, _)| {
            let lower = key.to_lowercase();
            !lower.starts_with("utm_")
                && !TRACKING.contains(&lower.as_str())
                && !host_params.contains(&lower.as_str())
        })
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    pairs.sort_by(|a, b| (a.0.to_lowercase(), &a.1).cmp(&(b.0.to_lowercase(), &b.1)));
    let query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&pairs)
        .finish();
    let mut key = format!("https://{netloc}{path}");
    if !query.is_empty() {
        key.push('?');
        key.push_str(&query)
    }
    key
}
static TITLE_FAMILIES: LazyLock<Vec<(Regex, Target)>> = LazyLock::new(|| {
    [
        (
            r"(?i)\b(ai|i\.?a\.?|intelig[êe]ncia artificial|llm|genai|machine learning|ml)\b",
            Target::AiLlm,
        ),
        (r"(?i)\b(dados|data)\b", Target::Data),
        (
            r"(?i)\b(tech lead|l[íi]der t[ée]cnico|staff|principal|arquitet\w*)\b",
            Target::TechLead,
        ),
        (
            r"(?i)\b(full ?stack|back ?end|software)\b",
            Target::FullStack,
        ),
    ]
    .into_iter()
    .filter_map(|(p, t)| Regex::new(p).ok().map(|r| (r, t)))
    .collect()
});
pub fn target_from_title(title: &str) -> Option<Target> {
    TITLE_FAMILIES
        .iter()
        .find(|(r, _)| r.is_match(title))
        .map(|(_, t)| *t)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn url_key_matches_the_crawler() {
        for (url, key) in [
            (
                "http://WWW.Example.com/jobs/1/",
                "https://example.com/jobs/1",
            ),
            (
                "https://example.com/a?utm_source=x&b=2&a=1#frag",
                "https://example.com/a?a=1&b=2",
            ),
            (
                "https://example.com:8080/a?gclid=1",
                "https://example.com:8080/a",
            ),
            (
                "https://boards.greenhouse.io/x/jobs/1?gh_src=abc",
                "https://boards.greenhouse.io/x/jobs/1",
            ),
            (
                "https://example.com/a?gh_src=keep",
                "https://example.com/a?gh_src=keep",
            ),
            (
                "https://example.com/a?q=caf%C3%A9+bar",
                "https://example.com/a?q=caf%C3%A9+bar",
            ),
            ("not a url", ""),
        ] {
            assert_eq!(url_key(url), key)
        }
    }
    #[test]
    fn target_comes_from_title_family() {
        for (t, target) in [
            ("Engenheiro de IA Sênior", Some(Target::AiLlm)),
            ("Data Engineer", Some(Target::Data)),
            ("Tech Lead Backend", Some(Target::TechLead)),
            ("Desenvolvedor Full Stack", Some(Target::FullStack)),
            ("Analista de Sistemas", None),
        ] {
            assert_eq!(target_from_title(t), target)
        }
    }
}
