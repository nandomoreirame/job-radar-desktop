//! robots.txt rules, as `job_radar/robots.py` (itself ported from ai-job-search,
//! MIT, Copyright (c) 2026 Mads Lorentzen). On equal specificity Allow wins
//! (RFC 9309 §2.2.2); blank lines inside a record do not end it.

use std::collections::BTreeMap;

use percent_encoding::percent_decode_str;

/// Sites that address the crawler by name keep applying to this app.
pub const PRODUCT_TOKEN: &str = "job-radar";

const DIRECTIVES: [&str; 6] = [
    "user-agent",
    "allow",
    "disallow",
    "sitemap",
    "crawl-delay",
    "host",
];

fn strip_comment(raw: &str) -> &str {
    raw.split('#').next().unwrap_or("").trim()
}

/// Does the body look like a robots.txt? An empty body is a valid allow-all; a
/// non-empty one without any known directive (an HTML page) is unreadable.
pub fn is_robots_body(text: &str) -> bool {
    if text.trim().is_empty() {
        return true;
    }
    text.lines().any(|raw| {
        let line = strip_comment(raw).to_lowercase();
        line.split_once(':')
            .is_some_and(|(name, _)| DIRECTIVES.contains(&name.trim()))
    })
}

type Rules = BTreeMap<String, Vec<(bool, String)>>;

fn parse_records(text: &str) -> (Rules, BTreeMap<String, f64>) {
    let mut rules: Rules = BTreeMap::new();
    let mut delays = BTreeMap::new();
    let mut agents: Vec<String> = Vec::new();
    let mut expect_agents = true;
    for raw in text.lines() {
        let line = strip_comment(raw);
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim().to_lowercase(), value.trim());
        match name.as_str() {
            "user-agent" => {
                if !expect_agents {
                    agents.clear();
                    expect_agents = true;
                }
                let agent = value.to_lowercase();
                rules.entry(agent.clone()).or_default();
                agents.push(agent);
            }
            "allow" | "disallow" if !agents.is_empty() => {
                expect_agents = false;
                for agent in &agents {
                    rules
                        .entry(agent.clone())
                        .or_default()
                        .push((name == "allow", value.to_owned()));
                }
            }
            "crawl-delay" if !agents.is_empty() => {
                expect_agents = false;
                let Ok(delay) = value.parse::<f64>() else {
                    continue;
                };
                for agent in &agents {
                    delays.insert(agent.clone(), delay);
                }
            }
            _ => {}
        }
    }
    (rules, delays)
}

/// Length of the pattern when it matches the path, `None` otherwise. The pattern is
/// percent-decoded so `/foo%20bar` still matches the decoded `/foo bar`.
fn match_length(pattern: &str, path: &str) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let pattern = percent_decode_str(pattern).decode_utf8_lossy().into_owned();
    let mut regex = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '$' => regex.push('$'),
            other => regex.push_str(&regex::escape(&other.to_string())),
        }
    }
    let matched = regex::Regex::new(&regex).is_ok_and(|re| re.is_match(path));
    matched.then(|| pattern.chars().count())
}

/// The most specific agent contained in the product token, else `*`.
fn pick_agent(rules: &Rules, product: &str) -> Option<String> {
    let product = product.to_lowercase();
    let best = rules
        .keys()
        .filter(|a| *a != "*" && product.contains(a.as_str()))
        .max_by_key(|a| a.len());
    match best {
        Some(agent) => Some(agent.clone()),
        None => rules.contains_key("*").then(|| "*".to_owned()),
    }
}

pub fn allowed(text: &str, product: &str, path: &str) -> bool {
    let (rules, _) = parse_records(text);
    let Some(agent) = pick_agent(&rules, product) else {
        return true;
    };
    let mut best: Option<(usize, bool)> = None;
    for (is_allow, pattern) in rules.get(&agent).into_iter().flatten() {
        let Some(length) = match_length(pattern, path) else {
            continue;
        };
        let better = match best {
            None => true,
            Some((len, _)) => length > len || (length == len && *is_allow),
        };
        if better {
            best = Some((length, *is_allow));
        }
    }
    best.is_none_or(|(_, allow)| allow)
}

pub fn crawl_delay(text: &str, product: &str) -> Option<f64> {
    let (rules, delays) = parse_records(text);
    delays.get(&pick_agent(&rules, product)?).copied()
}

/// The path robots rules see: percent-decoded, `/` when empty, query included.
pub fn path_of(url: &str) -> String {
    let Ok(parsed) = url::Url::parse(url) else {
        return "/".into();
    };
    let mut path = percent_decode_str(parsed.path())
        .decode_utf8_lossy()
        .into_owned();
    if path.is_empty() {
        path = "/".into();
    }
    if let Some(query) = parsed.query() {
        path.push('?');
        path.push_str(query);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_vectors() {
        let v: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/parity/robots.json")).unwrap();
        let product = v["product"].as_str().unwrap();
        for case in v["cases"].as_array().unwrap() {
            let body = case["body"].as_str().unwrap();
            assert_eq!(
                is_robots_body(body),
                case["is_robots"].as_bool().unwrap(),
                "{body}"
            );
            assert_eq!(crawl_delay(body, product), case["delay"].as_f64(), "{body}");
            for row in case["paths"].as_array().unwrap() {
                assert_eq!(
                    allowed(body, product, row[0].as_str().unwrap()),
                    row[1].as_bool().unwrap(),
                    "{body} {row}"
                );
            }
        }
    }
}
