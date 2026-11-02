# Plan: Crawler Core

> **Spec:** docs/specs/crawler-core/SPEC.md

**Goal:** Port the crawler's identity, filters, score, HTTP client and the Gupy and Solides connectors to Rust, so "Executar agora" finds real jobs and scores the imported ones.
**Architecture:** A `crawler/` module tree, blocking I/O on a worker thread, with pure parsing split from fetching. Parity with Python is proved by JSON vectors generated from the Python code and committed as fixtures. The pipeline talks to sources through a small trait, so its routing, rescoring and closed-ad rules are tested with fakes and no network.
**Tech Stack:** Rust 2024, ureq 3 (rustls), regex 1, fancy-regex 0.20, sha2, caseless, unicode-normalization, chrono, Tauri 2.12 events, React 19, Vitest 5.
**Total Tasks:** 10
**Estimated Complexity:** large
**Depends on:** `real-jobs-import` (nullable fields, `discard_reason`, `dismissed_jobs`, `insert_new_jobs`, `identity.rs`, `clock.rs`). Collides with it on `sqlite_store.rs`, `lib.rs`, `Sidebar.tsx`: implement it first.
**Lessons applied:** none

Conventions for every task:
- The reference is `../job-radar` (MIT, `nandomoreirame/job-radar`), next to this repo. **Port behavior exactly; do not improve a rule.** When Rust and a vector disagree, Python is right. Keep the comments that explain why a rule exists, translated to English.
- Rust commands run from `src-tauri/`. After each task: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`. `rpi-gates` only detects TypeScript here.
- No `unwrap()`/`expect()` outside tests, except compiling a constant regex inside `LazyLock`. Bound parameters only.
- Fixtures are recorded public pages copied from `../job-radar/tests/fixtures` or fictional text. Never commit real job data from the server, the personal profile, a server address or an absolute home path.
- No test touches the network. Verify runtime behavior by running the binary.

Batches: B1 = Tasks 1-3 (parity core), B2 = Tasks 4-6 (config, HTTP, connectors), B3 = Tasks 7-9 (store, pipeline, UI), B4 = Task 10 (docs and manual check).

---

## Checks

**Run rust**: `cd src-tauri && cargo test 2>&1`
**Run web**: `pnpm vitest run --reporter=verbose`

- C1 (REQ-001): fingerprint and URL key equal Python's on every vector. Proof: run rust | crawler::identity::tests::matches_the_python_vectors ... ok
- C2 (REQ-002): the verdict and reason of the hard filters equal Python's on every vector. Proof: run rust | crawler::filters::tests::matches_the_python_vectors ... ok
- C3 (REQ-002): the score equals Python's on every vector with a fixed clock. Proof: run rust | crawler::scoring::tests::matches_the_python_vectors ... ok
- C4 (REQ-002): skills extracted from a profile equal Python's. Proof: run rust | crawler::profile::tests::matches_the_python_skills ... ok
- C5 (REQ-003): a missing or skill-less profile refuses the run. Proof: run rust | crawler::config::tests::refuses_a_missing_or_empty_profile ... ok
- C6 (REQ-003): an invalid config names the field, and defaults are neutral. Proof: run rust | crawler::config::tests::reads_defaults_and_names_the_invalid_field ... ok
- C7 (REQ-004): the client honors robots, pacing and retries. Proof: run rust | crawler::http::tests::honors_robots_pacing_and_retries ... ok
- C8 (REQ-004): the client bounds the body and refuses other schemes. Proof: run rust | crawler::http::tests::bounds_the_body_and_refuses_other_schemes ... ok
- C9 (REQ-004): robots rules equal Python's. Proof: run rust | crawler::robots::tests::matches_the_python_vectors ... ok
- C10 (REQ-005): Gupy parsing equals Python's and a changed shape is an error. Proof: run rust | crawler::sources::gupy::tests::parses_the_recorded_search_like_python ... ok
- C11 (REQ-005): Solides parsing equals Python's. Proof: run rust | crawler::sources::solides::tests::parses_the_recorded_pages_like_python ... ok
- C12 (REQ-006): a crawl routes jobs by score, counts hard rejects and skips known ones. Proof: run rust | crawler::pipeline::tests::routes_new_jobs_and_skips_known_ones ... ok
- C13 (REQ-006): one failing source does not stop the others. Proof: run rust | crawler::pipeline::tests::a_failing_source_does_not_stop_the_others ... ok
- C14 (REQ-007): rescoring fills the fields and keeps user edits and stage. Proof: run rust | crawler::pipeline::tests::rescoring_keeps_user_edits_and_stage ... ok
- C15 (REQ-007): a stored description is scored with no request. Proof: run rust | crawler::pipeline::tests::a_stored_description_needs_no_request ... ok
- C16 (REQ-007): a rescored job below the cut stays where it is. Proof: run rust | crawler::pipeline::tests::a_low_rescore_stays_in_the_inbox ... ok
- C17 (REQ-008): closed pages go to Discarded; transient failures keep the card. Proof: run rust | crawler::pipeline::tests::closed_discards_and_failures_keep ... ok
- C18 (REQ-008): the circuit breaker stops a mass discard. Proof: run rust | crawler::pipeline::tests::circuit_breaker_stops_a_mass_discard ... ok
- C19 (REQ-008): Gupy's jobless page and Solides 404 count as closed, a block does not. Proof: run rust | crawler::sources::tests::detail_outcomes_follow_the_closed_rules ... ok
- C20 (REQ-009): a second start is refused while one runs. Proof: run rust | crawler::runner::tests::refuses_a_second_run ... ok
- C21 (REQ-009): the sidebar shows real status and disables the button while running. Proof: run web | ✓ CrawlerCard disables the button while running
- C22 (REQ-009): the last run survives a reopen. Proof: run rust | sqlite_store::tests::the_last_run_survives_a_reopen ... ok
- C23 (REQ-010): the sidebar explains a first run and a refused run. Proof: run web | ✓ CrawlerCard explains a never run and a refused run

---

### Task 1: Parity vectors and identity

**Requirement:** REQ-001
**Files:**
- Create: `scripts/gen_parity_vectors.py`
- Create: `src-tauri/tests/fixtures/parity/identity.json`
- Create: `src-tauri/src/crawler/mod.rs`
- Create: `src-tauri/src/crawler/identity.rs`
- From `real-jobs-import`, adjusted here only if a URL vector fails: `src-tauri/src/identity.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Create `scripts/gen_parity_vectors.py`. It runs the Python code on fictional inputs and writes the expected outputs; later tasks extend it:

```python
#!/usr/bin/env python3
"""Generates parity vectors from the Python crawler. Inputs are fictional.

Usage: python3 scripts/gen_parity_vectors.py [path-to-job-radar]
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT.parent / "job-radar")
sys.path.insert(0, str(REPO / "src"))
OUT = ROOT / "src-tauri" / "tests" / "fixtures" / "parity"

from job_radar.identity import company_key, job_fingerprint, title_key, url_key  # noqa: E402

TITLES = [
    "Backend Sr", "Sr Backend", "Engenheiro de Software Sênior - Remoto", "Desenvolvedor (Híbrido - São Paulo/SP)",
    "Dev Go - GO", "Analista - Goiânia - GO", "Tech Lead | Remote", "Coração de Dados / São Paulo", "Straße Developer",
    "Pessoa Desenvolvedora Back-end PL/SQL", "", "   ", "Staff AI Engineer (Remote)", "C++ Developer - Porto",
    "Engenheira de Machine Learning [Remoto]", "ＦＵＬＬ Stack",
]
COMPANIES = ["", "Acme", "ACME Ltda", "Grupo Acme S.A.", "Acme S/A", "Acme Inc", "ME", "Grupo", "Café & Cia LTDA ME", "Coração Labs EIRELI"]
URLS = [
    "http://WWW.Example.com/jobs/1/", "https://example.com/a?utm_source=x&b=2&a=1#frag", "https://example.com:8080/a?gclid=1",
    "https://example.com:443/a", "https://boards.greenhouse.io/x/jobs/1?gh_src=abc", "https://example.com/a?gh_src=keep",
    "https://x.lever.co/a/1?lever-source=li", "https://www.linkedin.com/jobs/view/1?trk=a&refId=b", "https://example.com/a?q=caf%C3%A9+bar",
    "https://example.com/?b=&a=1", "https://example.com/", "not a url", "", "ftp://example.com/a", "HTTPS://Example.COM/Path/",
]

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    identity = {
        "titles": [[t, title_key(t)] for t in TITLES],
        "companies": [[c, company_key(c)] for c in COMPANIES],
        "fingerprints": [[c, t, job_fingerprint(c, t)] for c in COMPANIES for t in TITLES],
        "urls": [[u, url_key(u)] for u in URLS],
    }
    (OUT / "identity.json").write_text(json.dumps(identity, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

if __name__ == "__main__":
    main()
```

Run it: `python3 scripts/gen_parity_vectors.py`. Commit the JSON.

Create `src-tauri/src/crawler/identity.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_vectors() {
        let vectors: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/identity.json")).unwrap();
        let rows = |key: &str| vectors[key].as_array().unwrap().clone();
        let text = |value: &Value| value.as_str().unwrap().to_owned();
        for row in rows("titles") {
            assert_eq!(title_key(&text(&row[0])), text(&row[1]), "title {row}");
        }
        for row in rows("companies") {
            assert_eq!(company_key(&text(&row[0])), text(&row[1]), "company {row}");
        }
        for row in rows("fingerprints") {
            assert_eq!(job_fingerprint(&text(&row[0]), &text(&row[1])), text(&row[2]), "fingerprint {row}");
        }
        for row in rows("urls") {
            assert_eq!(crate::identity::url_key(&text(&row[0])), text(&row[1]), "url {row}");
        }
        assert!(rows("fingerprints").len() > 100, "the vectors must not be empty");
    }
}
```

Add `pub mod crawler;` to `lib.rs` and `pub mod identity;` to `crawler/mod.rs`.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::identity 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `title_key`"

**Step 3: Write minimal implementation**

`Cargo.toml`: `sha2 = "0.10"`, `caseless = "0.2"`, `unicode-normalization = "0.1"`.

Prepend to `crawler/identity.rs`, porting `../job-radar/src/job_radar/identity.py` lines 26-110 (the constant lists, verbatim) and 132-250:

```rust
//! Job identity: must stay byte-identical to `job_radar/identity.py`, because
//! imported fingerprints decide whether a crawled job is already known. Errs on the
//! side of NOT merging: over-merging hides a job, under-merging only shows a duplicate.

use caseless::default_case_fold_str;
use sha2::{Digest, Sha256};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const COMPANY_SUFFIXES: [&str; 12] = ["ltda me", "ltda", "s a", "sa", "eireli", "epp", "inc", "llc", "ltd", "corp", "gmbh", "me"];

/// NFKD without marks, case folded, anything not `[0-9a-z]` becomes one space.
pub fn fold(text: &str) -> String {
    let plain: String = text.nfkd().filter(|c| !is_combining_mark(*c)).collect();
    let lowered = default_case_fold_str(&plain);
    let spaced: String = lowered.chars().map(|c| if c.is_ascii_digit() || c.is_ascii_lowercase() { c } else { ' ' }).collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn job_fingerprint(company: &str, title: &str) -> String {
    let tkey = title_key(title);
    let ckey = company_key(company);
    let joined = if ckey.is_empty() { tkey } else { format!("{ckey}|{tkey}") };
    let digest = Sha256::digest(joined.as_bytes());
    digest.iter().take(8).map(|byte| format!("{byte:02x}")).collect()
}
```

Port `title_key`, `company_key`, `_strip_trailing_tags`, `_segment_is_location_tag` and `_folded_endswith_city` line for line. `_TRAILING_SEP` and `_TRAILING_PAREN` use a lazy head and no lookaround, so `regex` works: `^(?s)(?P<head>.*?)\s*[-|/]\s*(?P<tail>[^-|/]+)$` and `^(?s)(?P<head>.*?)\s*\((?P<inner>[^()]*)\)\s*$`. `title_key` sorts a deduplicated word set.

Move nothing out of `crate::identity`: `url_key` and `target_from_title` stay there; adjust them only if a URL vector fails.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test identity 2>&1 | grep -E "identity::tests|test result: " | head -5`
Expected: PASS, `crawler::identity::tests::matches_the_python_vectors ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Job model and hard filters

**Requirement:** REQ-002
**Files:**
- Create: `src-tauri/src/crawler/job.rs`
- Create: `src-tauri/src/crawler/filters.rs`
- Create: `src-tauri/tests/fixtures/parity/filters.json`
- Modify: `scripts/gen_parity_vectors.py`
- Modify: `src-tauri/src/crawler/mod.rs`
- Modify: `src-tauri/Cargo.toml`

**Depends on:** Task 1.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Extend `gen_parity_vectors.py`. Each case sets the environment-free inputs, and the settings are passed explicitly in Rust:

```python
import os
from job_radar.filters import count_skill_hits, detect_contract, detect_seniority, detect_work_mode, parse_salary, passes_hard_filters
from job_radar.models import Job, clean_html

SKILLS = {"python", "aws", "typescript", "node.js", "c#x", "react", "ci/cd", "docker"}
JOBS = [
    ("Engenheiro de Software Sênior", "Python, AWS e TypeScript em produção.", "", "remote"),
    ("Desenvolvedor Pleno", "Python AWS Docker", "", "remote"),
    ("Dev PL/SQL Sênior", "Python AWS Docker", "", "remote"),
    ("Desenvolvedor Java Sênior", "Python AWS Docker", "", "remote"),
    ("Full Stack (React / Python / Go or Node.js or Java) Sênior", "AWS", "", "remote"),
    ("Senior Developer", "JavaScript, python, aws, docker. Sem java no texto? javascript.", "", "remote"),
    ("QA Engineer Sênior", "Python AWS Docker", "", "remote"),
    ("Product Engineer Sênior", "Python AWS Docker", "", "remote"),
    ("Product Owner Sênior", "Python AWS Docker", "", "remote"),
    ("Banco de Talentos Tecnologia", "Python AWS Docker", "", "remote"),
    ("Vaga afirmativa para mulheres - Dev Sênior", "Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Esta vaga é exclusiva para pessoas com deficiência. Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Vagas sinalizadas como exclusivas ou não para pessoas com deficiência. Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Trabalho presencial. Python AWS Docker", "São Paulo, SP", None),
    ("Dev Sênior", "Modelo híbrido. Python AWS Docker", "Campinas, SP", None),
    ("Dev Sênior", "Home office. Python AWS Docker", "", None),
    ("Dev Sênior", "Só Python", "", "remote"),
    ("Dev Sênior", "Python AWS Docker. Salário R$ 5.000,00 PJ", "", "remote"),
    ("Dev Sênior", "Python AWS Docker. VR de R$ 1.000,00", "", "remote"),
    ("Dev Sênior", "Python AWS Docker ci/cd node.js. 15k CLT", "", "remote"),
    ("Consultor SAP Sênior", "SAP ABAP SAP MM. Python AWS", "", "remote"),
    ("", "", "", None),
]
SETTINGS = {"MIN_SALARY_PJ": "10000", "MIN_SALARY_CLT": "9000", "COMMUTE_CITIES": "São Paulo", "MAX_JOB_AGE_DAYS": "0", "BLOCKED_COMPANIES": "Bloqueada S.A."}

def job_of(title, desc, loc, hint, company="Acme"):
    return Job(source="gupy", external_id="1", title=title, company=company, url="https://a.example/1", description=desc, location=loc, work_mode_hint=hint)

def filter_vectors():
    os.environ.update(SETTINGS)
    rows = []
    for title, desc, loc, hint in JOBS:
        job = job_of(title, desc, loc, hint)
        ok, reason = passes_hard_filters(job, SKILLS, 3)
        rows.append({"title": title, "description": desc, "location": loc, "hint": hint, "ok": ok, "reason": reason, "hits": sorted(count_skill_hits(job, SKILLS))})
    blocked = passes_hard_filters(job_of("Dev Sênior", "Python AWS Docker", "", "remote", company="BLOQUEADA"), SKILLS, 3)
    html = ["<p>Olá&nbsp;<b>mundo</b></p>", "&quot;a&quot; &amp; b", "", "a\n\n b"]
    return {
        "settings": SETTINGS, "skills": sorted(SKILLS), "jobs": rows, "blocked": list(blocked),
        "clean_html": [[h, clean_html(h)] for h in html],
        "salaries": [[t, parse_salary(t)] for t in ["R$ 14.000,00", "até 14k", "VR R$ 900", "R$ 14.000 a R$ 20.000", "ano de 2026", ""]],
        "contracts": [[t, detect_contract(t)] for t in ["vaga PJ", "CLT", "regime celetista", "nada", ""]],
    }
```

Add to `main()`: `(OUT / "filters.json").write_text(json.dumps(filter_vectors(), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")`.

Create `src-tauri/src/crawler/filters.rs` with:

```rust
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
        let skills: BTreeSet<String> = v["skills"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect();
        for row in v["jobs"].as_array().unwrap() {
            let job = Job::new("gupy", "1", row["title"].as_str().unwrap(), "Acme", "https://a.example/1")
                .description(row["description"].as_str().unwrap())
                .location(row["location"].as_str().unwrap())
                .work_mode_hint(row["hint"].as_str());
            let verdict = passes_hard_filters(&job, &skills, 3, &settings(), crate::clock::fixed_now());
            let ok = verdict.is_ok();
            let reason = verdict.err().unwrap_or_default();
            assert_eq!(ok, row["ok"].as_bool().unwrap(), "{row}");
            assert_eq!(reason, row["reason"].as_str().unwrap(), "{row}");
            let hits: Vec<String> = count_skill_hits(&job, &skills).into_iter().collect();
            let expected: Vec<String> = row["hits"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect();
            assert_eq!(hits, expected, "{row}");
        }
        for row in v["salaries"].as_array().unwrap() {
            assert_eq!(parse_salary(row[0].as_str().unwrap()), row[1].as_u64().map(|n| n as u32), "{row}");
        }
        for row in v["contracts"].as_array().unwrap() {
            assert_eq!(detect_contract(row[0].as_str().unwrap()).map(str::to_owned).as_deref(), row[1].as_str(), "{row}");
        }
        assert!(v["jobs"].as_array().unwrap().iter().any(|r| !r["ok"].as_bool().unwrap()));
        assert!(v["jobs"].as_array().unwrap().iter().any(|r| r["ok"].as_bool().unwrap()));
    }

    #[test]
    fn blocks_a_company_by_key() {
        let skills: BTreeSet<String> = ["python", "aws", "docker"].map(str::to_owned).into_iter().collect();
        let job = Job::new("gupy", "1", "Dev Sênior", "BLOQUEADA", "https://a.example/1").description("Python AWS Docker").work_mode_hint(Some("remote"));
        assert_eq!(
            passes_hard_filters(&job, &skills, 3, &settings(), crate::clock::fixed_now()),
            Err(vectors()["blocked"][1].as_str().unwrap().to_owned())
        );
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::filters 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find type `FilterSettings`"

**Step 3: Write minimal implementation**

`Cargo.toml`: `fancy-regex = "0.20"`, `chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }`.

`src-tauri/src/clock.rs`: add `pub type Now = chrono::DateTime<chrono::FixedOffset>;`, `pub fn now() -> Now`, and `#[cfg(test)] pub fn fixed_now() -> Now` returning 2026-11-02T12:00:00-03:00.

`crawler/job.rs` ports `models.py`:

```rust
//! The job shape shared by every source, as in `job_radar/models.py`.

use std::sync::LazyLock;

use regex::Regex;

use crate::clock::Now;
use crate::crawler::identity::job_fingerprint;

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// Plain text out of a source's HTML; filters need text, not markup.
pub fn clean_html(raw: &str) -> String {
    let text = TAG.replace_all(raw, " ");
    let text = html_escape::decode_html_entities(&text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Job {
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub company: String,
    pub url: String,
    pub description: String,
    pub location: String,
    pub work_mode_hint: Option<String>,
    pub tags: Vec<String>,
    pub published_at: Option<Now>,
}

impl Job {
    pub fn new(source: &str, external_id: &str, title: &str, company: &str, url: &str) -> Self {
        Self {
            source: source.to_owned(),
            external_id: external_id.to_owned(),
            title: title.trim().to_owned(),
            company: company.trim().to_owned(),
            url: url.to_owned(),
            ..Self::default()
        }
    }
    pub fn description(mut self, raw: &str) -> Self {
        self.description = clean_html(raw);
        self
    }
    pub fn location(mut self, value: &str) -> Self {
        self.location = value.trim().to_owned();
        self
    }
    pub fn work_mode_hint(mut self, value: Option<&str>) -> Self {
        self.work_mode_hint = value.map(str::to_owned);
        self
    }
    pub fn fingerprint(&self) -> String {
        job_fingerprint(&self.company, &self.title)
    }
    /// The one text the filters search, lowercased like Python's casefold.
    pub fn haystack(&self) -> String {
        let tags = self.tags.join(" ");
        let parts = [self.title.as_str(), &self.company, &self.description, &tags, &self.location];
        let joined = parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ");
        caseless::default_case_fold_str(&joined)
    }
}
```

Add `html-escape = "0.2"` to `Cargo.toml`. Confirm `haystack` part order against `models.py:88-92` (the output above is cut at column 90).

`crawler/filters.rs` ports `filters.py` fully, in the same order inside `passes_hard_filters`. Return `Result<(), String>`, where the error is the same pt-BR reason as Python. `FilterSettings` replaces `settings.py`'s environment reads:

```rust
#[derive(Debug, Clone, Default)]
pub struct FilterSettings {
    pub min_salary_pj: u32,
    pub min_salary_clt: u32,
    pub commute_cities: BTreeSet<String>,
    pub max_job_age_days: u32,
    pub blocked_companies: BTreeSet<String>,
}
```

Normalize `commute_cities` (accents stripped, case folded) and `blocked_companies` (`company_key`) when comparing, as `settings.py` does. Regexes with lookaround use `fancy_regex::Regex`: `_BELOW_RE` (`pl\.?\b(?!/?sql)`) and the skill match. `count_skill_hits` compiles nothing per job: for each skill find its occurrences in `haystack` with `str::match_indices` and check the neighbors by hand, matching `(?<![a-z0-9+#.])skill(?![a-z0-9+#])`. Return a `BTreeSet<String>`. Port the salary formatting `R$ 5.000` exactly, including the `.replace(",", ".")`.

`re.I` becomes `(?i)` in each pattern, and Python's `\w`/`\b` are Unicode as in both Rust crates. Any vector mismatch here is a port error to fix in Rust.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::filters 2>&1 | grep -E "filters::tests|test result: " | head -4`
Expected: PASS, both tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: Score and profile

**Requirement:** REQ-002
**Files:**
- Create: `src-tauri/src/crawler/scoring.rs`
- Create: `src-tauri/src/crawler/profile.rs`
- Create: `src-tauri/tests/fixtures/parity/scoring.json`
- Create: `src-tauri/tests/fixtures/perfil.md`
- Modify: `scripts/gen_parity_vectors.py`
- Modify: `src-tauri/src/crawler/mod.rs`

**Depends on:** Task 2.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Copy `../job-radar/tests/fixtures/perfil.md` to `src-tauri/tests/fixtures/perfil.md` after confirming it holds fictional text; replace anything personal with a fictional value before generating.

Extend `gen_parity_vectors.py`. The clock is frozen by computing the expected age adjustment from a fixed `published_at` relative to the real today in Python, so freeze `datetime` in the `scoring` module with a small subclass:

```python
import datetime as dt
import job_radar.scoring as scoring
from job_radar.profile import expand_synonyms, extract_skills, load_profile

class FixedDatetime(dt.datetime):
    @classmethod
    def now(cls, tz=None):
        return cls(2026, 11, 2, 15, 0, 0, tzinfo=dt.timezone.utc).astimezone(tz) if tz else cls(2026, 11, 2, 12, 0, 0)

def scoring_vectors():
    scoring.datetime = FixedDatetime
    utc = dt.timezone.utc
    profile = (ROOT / "src-tauri" / "tests" / "fixtures" / "perfil.md").read_text(encoding="utf-8")
    skills = expand_synonyms(extract_skills(profile))
    dates = [None, dt.datetime(2026, 11, 1, tzinfo=utc), dt.datetime(2026, 10, 5, tzinfo=utc), dt.datetime(2026, 9, 1, tzinfo=utc)]
    rows = []
    for title, desc, loc, hint in JOBS:
        for published in dates:
            job = job_of(title, desc, loc, hint)
            job.published_at = published
            hits = count_skill_hits(job, skills)
            rows.append({"title": title, "description": desc, "location": loc, "hint": hint,
                         "published_at": published.isoformat() if published else None,
                         "hits": len(hits), "score": scoring.score_job(job, skills, hits)})
    return {"skills": sorted(skills), "extracted": sorted(extract_skills(profile)), "jobs": rows}
```

`src-tauri/src/crawler/scoring.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::filters::count_skill_hits;
    use crate::crawler::job::Job;
    use serde_json::Value;
    use std::collections::BTreeSet;

    #[test]
    fn matches_the_python_vectors() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/scoring.json")).unwrap();
        let skills: BTreeSet<String> = v["skills"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect();
        let rows = v["jobs"].as_array().unwrap();
        assert!(rows.len() > 50);
        for row in rows {
            let mut job = Job::new("gupy", "1", row["title"].as_str().unwrap(), "Acme", "https://a.example/1")
                .description(row["description"].as_str().unwrap())
                .location(row["location"].as_str().unwrap())
                .work_mode_hint(row["hint"].as_str());
            job.published_at = row["published_at"].as_str().map(|d| chrono::DateTime::parse_from_rfc3339(d).unwrap());
            let hits = count_skill_hits(&job, &skills);
            assert_eq!(hits.len() as u64, row["hits"].as_u64().unwrap(), "{row}");
            assert_eq!(u64::from(score_job(&job, hits.len(), crate::clock::fixed_now())), row["score"].as_u64().unwrap(), "{row}");
        }
    }
}
```

`src-tauri/src/crawler/profile.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_skills() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/scoring.json")).unwrap();
        let list = |key: &str| -> BTreeSet<String> { v[key].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_owned()).collect() };
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
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::scoring crawler::profile 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `score_job`"

**Step 3: Write minimal implementation**

`scoring.rs` ports `scoring.py` with `score_job(job: &Job, hits: usize, now: Now) -> u8`: Python's `skills` argument is unused, so drop it. Compute in `i32` and clamp to 0-100. Age is `(now - published).num_days()`, floored like Python's `timedelta.days`: use `(now - published).num_seconds().div_euclid(86_400)`. `TARGET_PATTERNS` takes the maximum matching weight.

`profile.rs` ports `profile.py`: `_STOPWORDS`, `_SYNONYMS`, `extract_skills`, `expand_synonyms` return `BTreeSet<String>`, and `load_profile(path) -> io::Result<BTreeSet<String>>`. The trailing version regex is `\s+\d+(\.\d+)?\+?$`. Python's `.strip("*` ")` trims those three characters.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler 2>&1 | grep -E "scoring::tests|profile::tests|test result: " | head -5`
Expected: PASS, every test `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: crawler.json

**Requirement:** REQ-003
**Files:**
- Create: `src-tauri/src/crawler/config.rs`
- Modify: `src-tauri/src/crawler/mod.rs`

**Depends on:** Task 3.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

```rust
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
        let path = write(dir.path(), "crawler.json", &serde_json::json!({ "profile_path": profile }).to_string());
        let config = load_config(&path).unwrap();
        assert_eq!((config.min_score, config.min_skills), (70, 3));
        assert_eq!(config.filters.min_salary_pj, 0);
        assert!(config.filters.commute_cities.is_empty() && config.filters.blocked_companies.is_empty());

        let bad = write(dir.path(), "bad.json", &serde_json::json!({ "profile_path": profile, "min_score": 500 }).to_string());
        assert_eq!(load_config(&bad).unwrap_err(), ConfigError::Invalid("min_score".into()));
        let not_json = write(dir.path(), "x.json", "{ nope");
        assert!(matches!(load_config(&not_json), Err(ConfigError::Unreadable(_))));
    }

    #[test]
    fn refuses_a_missing_or_empty_profile() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_config(&dir.path().join("none.json")).unwrap_err(), ConfigError::Missing);
        let missing = write(dir.path(), "a.json", &serde_json::json!({ "profile_path": dir.path().join("none.md") }).to_string());
        assert!(matches!(load_crawl_setup(&missing), Err(ConfigError::ProfileUnreadable)));
        let empty_cv = write(dir.path(), "cv.md", "# CV sem a seção\n");
        let empty = write(dir.path(), "b.json", &serde_json::json!({ "profile_path": empty_cv }).to_string());
        assert_eq!(load_crawl_setup(&empty).unwrap_err(), ConfigError::NoSkills);
        let good_cv = write(dir.path(), "ok.md", PROFILE);
        let good = write(dir.path(), "c.json", &serde_json::json!({ "profile_path": good_cv }).to_string());
        assert!(load_crawl_setup(&good).unwrap().skills.contains("python"));
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::config 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `load_config`"

**Step 3: Write minimal implementation**

```rust
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
fn default_min_score() -> u32 { 70 }
fn default_min_skills() -> u32 { 3 }

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
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(ConfigError::Missing),
        Err(error) => return Err(ConfigError::Unreadable(error.to_string())),
    };
    let raw: RawConfig = serde_json::from_str(&text).map_err(|e| ConfigError::Unreadable(e.to_string()))?;
    let min_score = u8::try_from(raw.min_score).ok().filter(|v| *v <= 100).ok_or_else(|| ConfigError::Invalid("min_score".into()))?;
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
```

Task 9 turns each `ConfigError` into a translated sidebar message. `deny_unknown_fields` is dropped in `crawler-sources` only if it adds fields without declaring them; it should declare them.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::config 2>&1 | grep -E "config::tests|test result: " | head -3`
Expected: PASS, both tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: Polite HTTP client with robots

**Requirement:** REQ-004
**Files:**
- Create: `src-tauri/src/crawler/robots.rs`
- Create: `src-tauri/src/crawler/http.rs`
- Create: `src-tauri/tests/fixtures/parity/robots.json`
- Modify: `scripts/gen_parity_vectors.py`
- Modify: `src-tauri/src/crawler/mod.rs`
- Modify: `src-tauri/Cargo.toml`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Vectors in `gen_parity_vectors.py`:

```python
from job_radar.robots import PRODUCT_TOKEN, allowed, crawl_delay_from_body, is_robots_body

def robots_vectors():
    bodies = [
        "User-agent: *\nDisallow: /private\nAllow: /private/ok\nCrawl-delay: 2\n",
        "User-agent: job-radar\nDisallow: /\n\nUser-agent: *\nAllow: /\n",
        "User-agent: *\nDisallow: /*.pdf$\nDisallow: /a*b\n",
        "User-agent: *\nDisallow:\n",
        "# only comments\n",
        "<html>not robots</html>",
        "",
    ]
    paths = ["/", "/private", "/private/ok", "/private/x", "/doc.pdf", "/doc.pdf?x=1", "/axxb", "/a"]
    return {"product": PRODUCT_TOKEN, "cases": [
        {"body": b, "is_robots": is_robots_body(b), "delay": crawl_delay_from_body(b, PRODUCT_TOKEN),
         "paths": [[p, allowed(b, PRODUCT_TOKEN, p)] for p in paths]} for b in bodies]}
```

`src-tauri/src/crawler/robots.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_vectors() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/robots.json")).unwrap();
        let product = v["product"].as_str().unwrap();
        for case in v["cases"].as_array().unwrap() {
            let body = case["body"].as_str().unwrap();
            assert_eq!(is_robots_body(body), case["is_robots"].as_bool().unwrap(), "{body}");
            assert_eq!(crawl_delay(body, product), case["delay"].as_f64(), "{body}");
            for row in case["paths"].as_array().unwrap() {
                assert_eq!(allowed(body, product, row[0].as_str().unwrap()), row[1].as_bool().unwrap(), "{body} {row}");
            }
        }
    }
}
```

`src-tauri/src/crawler/http.rs` tests, with a fake transport so nothing touches the network:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    #[derive(Default)]
    struct Fake {
        responses: RefCell<VecDeque<(String, Result<RawResponse, String>)>>,
        requests: RefCell<Vec<String>>,
        slept: RefCell<Vec<f64>>,
    }
    impl Fake {
        fn on(self, url: &str, status: u16, body: &str) -> Self {
            self.responses.borrow_mut().push_back((url.to_owned(), Ok(RawResponse { status, body: body.as_bytes().to_vec(), retry_after: None })));
            self
        }
    }
    impl Transport for &Fake {
        fn get(&self, url: &str, _accept: &str) -> Result<RawResponse, String> {
            self.requests.borrow_mut().push(url.to_owned());
            let mut queue = self.responses.borrow_mut();
            let at = queue.iter().position(|(u, _)| u == url).ok_or_else(|| format!("unexpected {url}"))?;
            queue.remove(at).map(|(_, r)| r).unwrap()
        }
        fn sleep(&self, seconds: f64) {
            self.slept.borrow_mut().push(seconds);
        }
    }

    #[test]
    fn honors_robots_pacing_and_retries() {
        let fake = Fake::default()
            .on("https://a.example/robots.txt", 200, "User-agent: *\nDisallow: /no\nCrawl-delay: 3\n")
            .on("https://a.example/one", 503, "")
            .on("https://a.example/one", 200, "ok")
            .on("https://a.example/two", 200, "ok");
        let client = Client::new(&fake);
        assert_eq!(client.get("https://a.example/no", GetOptions::default()), Err(HttpError::Blocked));
        assert_eq!(client.get("https://a.example/one", GetOptions::default()).unwrap().body, b"ok");
        assert_eq!(client.get("https://a.example/two", GetOptions::default()).unwrap().body, b"ok");
        assert!(fake.slept.borrow().iter().any(|s| *s >= 3.0), "crawl-delay not honored: {:?}", fake.slept.borrow());
        assert_eq!(fake.requests.borrow().iter().filter(|u| u.ends_with("robots.txt")).count(), 1);
        // 404 is not retried and is an answer, not an error of the client.
        let gone = Fake::default().on("https://b.example/robots.txt", 404, "").on("https://b.example/x", 404, "");
        assert_eq!(Client::new(&gone).get("https://b.example/x", GetOptions::default()), Err(HttpError::Status(404)));
    }

    #[test]
    fn bounds_the_body_and_refuses_other_schemes() {
        let fake = Fake::default();
        let client = Client::new(&fake);
        assert_eq!(client.get("file:///etc/passwd", GetOptions::default()), Err(HttpError::BadUrl));
        assert_eq!(client.get("ftp://a.example/x", GetOptions::default()), Err(HttpError::BadUrl));
        assert!(fake.requests.borrow().is_empty());
        let big = "x".repeat(MAX_BODY + 1);
        let fake = Fake::default().on("https://c.example/robots.txt", 404, "").on("https://c.example/x", 200, &big);
        assert_eq!(Client::new(&fake).get("https://c.example/x", GetOptions::default()), Err(HttpError::TooLarge));
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::http crawler::robots 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find trait `Transport`"

**Step 3: Write minimal implementation**

`Cargo.toml`: `ureq = { version = "3", default-features = false, features = ["rustls", "gzip"] }`.

`robots.rs` ports the pure part of `robots.py` (`is_robots_body`, `_parse_records`, `_match_length`, `_pick_agent`, `select_rules`, `allowed`, `crawl_delay_from_body`), with `PRODUCT_TOKEN = "job-radar"`, so sites that address the crawler by name keep applying.

`http.rs` ports `http.py` plus `RobotsCache`:

```rust
//! Every request goes through here: identity, robots, pause per host, retry, limits.
//! The transport is a trait so tests inject responses and never touch the network.

pub const MAX_BODY: usize = 10 * 1024 * 1024;
pub const USER_AGENT: &str = concat!("job-radar-desktop/", env!("CARGO_PKG_VERSION"), " (+https://github.com/nandomoreirame/job-radar-desktop)");
const MIN_PAUSE: f64 = 1.0;
const RETRIES: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawResponse { pub status: u16, pub body: Vec<u8>, pub retry_after: Option<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response { pub body: Vec<u8> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    BadUrl,
    /// robots.txt disallows the page, or its robots.txt cannot be read.
    Blocked,
    Status(u16),
    TooLarge,
    Network(String),
}

pub trait Transport {
    fn get(&self, url: &str, accept: &str) -> Result<RawResponse, String>;
    fn sleep(&self, seconds: f64);
}

#[derive(Debug, Clone, Default)]
pub struct GetOptions<'a> {
    pub accept: Option<&'a str>,
    /// Extra statuses to retry, as Solides needs for its 403 bursts.
    pub retry_statuses: &'a [u16],
    /// Fixed backoff per attempt instead of the exponential default.
    pub fixed_backoff: Option<f64>,
    pub retries: Option<u32>,
}
```

`Client<T: Transport>` holds a `RefCell<HashMap<String, HostPolicy>>` and the last request instant per host; `Client` is created inside the worker thread and is not shared. Rules, from `http.py` and `robots.py`:

- Scheme other than http(s) gives `BadUrl` before any request.
- 4xx on robots.txt allows all (RFC 9309); 5xx or network retried up to 2 times, then the host is unreachable and every URL gives `Blocked`, as `allowed` does in Python; a body that fails `is_robots_body` is unreachable too.
- Wait `max(MIN_PAUSE, crawl_delay)` since the last request to that host.
- Retry 429, 5xx, network errors and `retry_statuses`; honor `Retry-After` up to 60 s; the default backoff is Python's `_backoff_seconds`, with jitter from a fixed value in tests.
- Other statuses give `Status(code)` at once.
- A body above `MAX_BODY` gives `TooLarge`.

`UreqTransport` implements `Transport` with `ureq::Agent`: timeout 25 s, `max_redirects(5)`, `http_status_as_error(false)`, the `User-Agent`, `read_to_vec` with `limit(MAX_BODY + 1)`. Redirects to non-http(s) schemes are refused by ureq. Sends no cookie, no credential and no user data.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler 2>&1 | grep -E "http::tests|robots::tests|test result: " | head -5`
Expected: PASS, three tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: Gupy and Solides connectors

**Requirement:** REQ-005, REQ-008
**Files:**
- Create: `src-tauri/src/crawler/sources/mod.rs`
- Create: `src-tauri/src/crawler/sources/gupy.rs`
- Create: `src-tauri/src/crawler/sources/solides.rs`
- Create: `src-tauri/tests/fixtures/gupy_search.html`
- Create: `src-tauri/tests/fixtures/gupy_job.html`
- Create: `src-tauri/tests/fixtures/gupy_closed.html`
- Create: `src-tauri/tests/fixtures/solides_listing.json`
- Create: `src-tauri/tests/fixtures/solides_jobposting.html`
- Create: `src-tauri/tests/fixtures/parity/sources.json`
- Modify: `scripts/gen_parity_vectors.py`
- Modify: `src-tauri/src/crawler/mod.rs`

**Depends on:** Tasks 2 and 5.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Copy the three recorded fixtures from `../job-radar/tests/fixtures`. Write `gupy_job.html` and `gupy_closed.html` by hand, fictional, with the shape measured on 2026-11-02: a `<script id="__NEXT_DATA__">` whose `props.pageProps.job` has `id`, `name`, `description`, `responsibilities`, `prerequisites`, `publishedAt`, `addressCity`, `addressStateShortName`, `workplaceType`; the closed page's `pageProps` holds only `_nextI18Next`, `toggles`, `userId`, `userLanguage`. Before writing them, fetch one live Gupy job with a browser or `curl` and confirm the field names of `workplaceType` and the address, which the measurement cut off; record what you find in `STATE.md`.

Vectors in `gen_parity_vectors.py`, using the Python parsers on the same fixtures:

```python
from job_radar.sources import gupy as py_gupy, solides as py_solides

def job_dict(j):
    return {"source": j.source, "external_id": j.external_id, "title": j.title, "company": j.company, "url": j.url,
            "description": j.description, "location": j.location, "hint": j.work_mode_hint, "tags": j.tags,
            "published_at": j.published_at.isoformat() if j.published_at else None, "fingerprint": j.fingerprint}

def source_vectors():
    fixtures = ROOT / "src-tauri" / "tests" / "fixtures"
    search = (fixtures / "gupy_search.html").read_text(encoding="utf-8")
    listing = json.loads((fixtures / "solides_listing.json").read_text(encoding="utf-8"))
    posting = (fixtures / "solides_jobposting.html").read_text(encoding="utf-8")
    meta, solides = py_solides.listing_from_page(json.dumps(listing) if '"initialData"' in json.dumps(listing) else '"initialData":' + json.dumps(listing))
    return {"gupy": [job_dict(py_gupy.parse_job(i)) for i in py_gupy.listing_from_page(search)],
            "solides_meta": meta, "solides": [job_dict(j) for j in solides],
            "solides_posting": py_solides.clean_html(py_solides.parse_jobposting(posting))}
```

Check how `../job-radar/tests/test_solides.py` feeds `solides_listing.json` into `listing_from_page` and use the same call.

`src-tauri/src/crawler/sources/gupy.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::assert_jobs_match;

    #[test]
    fn parses_the_recorded_search_like_python() {
        let jobs = parse_search(include_str!("../../../tests/fixtures/gupy_search.html")).unwrap();
        assert_jobs_match(&jobs, "gupy");
        assert!(!jobs.is_empty());
    }

    #[test]
    fn a_page_without_the_payload_is_an_error() {
        assert!(parse_search("<html>nothing</html>").is_err());
        assert!(parse_search("<script id=\"__NEXT_DATA__\">{ broken</script>").is_err());
    }

    #[test]
    fn reads_a_job_page() {
        let detail = parse_detail(include_str!("../../../tests/fixtures/gupy_job.html")).unwrap().unwrap();
        assert!(detail.description.contains("Python"));
        assert!(detail.published_at.is_some());
    }

    #[test]
    fn a_page_without_a_job_is_closed() {
        assert_eq!(parse_detail(include_str!("../../../tests/fixtures/gupy_closed.html")), Ok(None));
        assert!(parse_detail("<html>no payload at all</html>").is_err());
    }
}
```

`src-tauri/src/crawler/sources/solides.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::{assert_jobs_match, vectors};

    #[test]
    fn parses_the_recorded_pages_like_python() {
        let listing = include_str!("../../../tests/fixtures/solides_listing.json");
        let (_, jobs) = listing_from_blob(&format!("\"initialData\":{listing}")).unwrap();
        assert_jobs_match(&jobs, "solides");
        let posting = parse_jobposting(include_str!("../../../tests/fixtures/solides_jobposting.html"));
        assert_eq!(posting, vectors()["solides_posting"].as_str().unwrap());
    }

    #[test]
    fn keeps_a_job_without_a_date_and_drops_an_old_one() {
        let mut job = crate::crawler::job::Job::new("solides", "1", "Dev", "Acme", "https://x");
        assert!(is_fresh(&job, 45, crate::clock::fixed_now()));
        job.published_at = Some(chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00+00:00").unwrap());
        assert!(!is_fresh(&job, 45, crate::clock::fixed_now()));
    }

    #[test]
    fn decodes_pushes_with_escaped_quotes() {
        let html = r#"<script>self.__next_f.push([1,"a\"b"])</script><script>self.__next_f.push([1,"c"])</script>"#;
        assert_eq!(rsc_blob(html), "a\"bc");
    }
}
```

`src-tauri/src/crawler/sources/mod.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::http::HttpError;

    #[test]
    fn detail_outcomes_follow_the_closed_rules() {
        let gupy = include_str!("../../../tests/fixtures/gupy_closed.html");
        assert_eq!(gupy::outcome_of(Ok(gupy.as_bytes().to_vec())), DetailOutcome::Closed);
        let solides = solides::outcome_of(Err(HttpError::Status(404)));
        assert_eq!(solides, DetailOutcome::Closed);
        assert_eq!(gupy::outcome_of(Err(HttpError::Status(410))), DetailOutcome::Closed);
        for transient in [HttpError::Blocked, HttpError::Status(403), HttpError::Status(500), HttpError::Network("x".into()), HttpError::TooLarge] {
            assert!(matches!(solides::outcome_of(Err(transient.clone())), DetailOutcome::Failed(_)), "{transient:?}");
            assert!(matches!(gupy::outcome_of(Err(transient.clone())), DetailOutcome::Failed(_)), "{transient:?}");
        }
        // No JSON-LD on a 200 is a parse failure, not a closed ad.
        assert!(matches!(solides::outcome_of(Ok(b"<html></html>".to_vec())), DetailOutcome::Failed(_)));
        assert!(matches!(gupy::outcome_of(Ok(b"<html></html>".to_vec())), DetailOutcome::Failed(_)));
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::sources 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `parse_search`"

**Step 3: Write minimal implementation**

`sources/mod.rs`:

```rust
//! One connector per file: pure parsing tested on recorded pages, plus fetching.

pub mod gupy;
pub mod solides;

use crate::crawler::http::{Client, Transport};
use crate::crawler::filters::FilterSettings;
use crate::crawler::job::Job;

/// What a job's page says. Only positive evidence closes a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailOutcome {
    Found(JobDetail),
    Closed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobDetail {
    pub description: String,
    pub location: String,
    pub work_mode_hint: Option<String>,
    pub published_at: Option<crate::clock::Now>,
}

/// `keep` decides which listed jobs deserve a detail request, as in `collect.py`.
pub trait Source {
    fn name(&self) -> &'static str;
    /// Owner of a stored `source` value such as `gupy` or `github:backend-br`.
    fn owns(&self, source: &str) -> bool;
    fn collect(&self, keep: &dyn Fn(&Job) -> bool, progress: &dyn Fn(&str)) -> Result<Vec<Job>, String>;
    fn fetch_detail(&self, url: &str) -> DetailOutcome;
}

#[cfg(test)]
pub(crate) mod tests_support {
    use crate::crawler::job::Job;
    use serde_json::Value;

    pub fn vectors() -> Value {
        serde_json::from_str(include_str!("../../../tests/fixtures/parity/sources.json")).unwrap()
    }

    pub fn assert_jobs_match(jobs: &[Job], key: &str) {
        let v = vectors();
        let expected = v[key].as_array().unwrap();
        assert_eq!(jobs.len(), expected.len(), "{key}");
        for (job, row) in jobs.iter().zip(expected) {
            for (actual, field) in [
                (&job.external_id, "external_id"), (&job.title, "title"), (&job.company, "company"),
                (&job.url, "url"), (&job.description, "description"), (&job.location, "location"),
            ] {
                assert_eq!(actual, row[field].as_str().unwrap(), "{key} {field}");
            }
            assert_eq!(job.work_mode_hint.as_deref(), row["hint"].as_str(), "{key}");
            assert_eq!(job.fingerprint(), row["fingerprint"].as_str().unwrap(), "{key}");
        }
    }
}
```

`gupy.rs` ports `sources/gupy.py` (`SEARCH`, the term in the PATH, 12 items per term, the `__NEXT_DATA__` regex) and `targets.py`'s `TARGETS` terms. New on top of Python: `parse_detail(html) -> Result<Option<JobDetail>, String>`. The description joins `description`, `responsibilities` and `prerequisites` through `clean_html`, skipping empties. `Ok(None)` when the payload parses and `pageProps.job` is absent; `Err` when `__NEXT_DATA__` is missing or the job has an empty description. `outcome_of(result)` applies the closed rules: `Status(404|410)` and `Ok(None)` close, the rest is `Failed`.

`solides.rs` ports `sources/solides.py`: `rsc_blob` (decode each push with `serde_json::Deserializer::from_str(...).into_iter::<String>()`, so escaped quotes work), `_slice_json`, `parse_listing`, `is_fresh`, `listing_from_blob`, `parse_jobposting`; the URL is built from the id, never from `redirectLink`. Retries use `GetOptions { retry_statuses: &[403, 429], fixed_backoff: Some(5.0), retries: Some(2), .. }`. Delays are 1.2 s between pages and 0.8 s between details through `Transport::sleep`. `outcome_of` closes on 404 or 410 only. `validThrough` is ignored on purpose (spec Verified Claims).

Connector structs take `Rc<Client<T>>` so both share pacing and robots within a run.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::sources 2>&1 | grep -E "sources::|test result: " | head -12`
Expected: PASS, every test `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: Store support for crawling

**Requirement:** REQ-006, REQ-007, REQ-009
**Files:**
- Create: `src-tauri/migrations/003_crawl_runs.sql`
- Modify: `src-tauri/src/sqlite_store.rs`
- Modify: `src-tauri/src/domain.rs`

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, security-reviewer, business-logic-reviewer

**Step 1: Write the failing test**

Add to `sqlite_store.rs` tests:

```rust
    #[test]
    fn the_last_run_survives_a_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            assert_eq!(store.last_run().unwrap(), None);
            let id = store.start_run("2026-11-02T10:00:00-03:00").unwrap();
            assert!(store.last_run().unwrap().unwrap().finished_at.is_none());
            store.finish_run(id, "2026-11-02T10:01:00-03:00", r#"{"found":3}"#).unwrap();
        }
        let run = SqliteStore::open(&path).unwrap().last_run().unwrap().unwrap();
        assert_eq!(run.finished_at.as_deref(), Some("2026-11-02T10:01:00-03:00"));
        assert_eq!(run.stats, r#"{"found":3}"#);
    }

    #[test]
    fn applying_a_score_keeps_user_fields() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.score = None;
        card.notes = "minha nota".into();
        store.seed_if_empty(std::slice::from_ref(&card)).unwrap();
        store.move_card(&card.id, Stage::Triage, None).unwrap();
        let scoring = ScoreUpdate { score: 85, target: None, work_model: Some(crate::domain::WorkModel::Hybrid), location: Some("Recife".into()), description: "texto".into(), url_key: None };
        store.apply_score(&card.id, &scoring).unwrap();
        let stored = store.list_cards().unwrap().remove(0);
        assert_eq!((stored.score, stored.stage, stored.notes.as_str()), (Some(85), Stage::Triage, "minha nota"));
        assert_eq!(stored.work_model, Some(crate::domain::WorkModel::Hybrid));
        // Target already known stays; a null update never erases data.
        assert_eq!(stored.target, card.target);
    }

    #[test]
    fn lists_only_cards_needing_a_score() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut cards = crate::mock_data::sample_cards();
        cards[0].score = None;
        store.seed_if_empty(&cards).unwrap();
        store.discard_card(&cards[1].id, "x", "2026-11-02").unwrap();
        let pending = store.unscored_cards().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, cards[0].id);
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | tail -4`
Expected: FAIL to compile with "no method named `last_run`"

**Step 3: Write minimal implementation**

`003_crawl_runs.sql`: the table from the spec. Add it to `MIGRATIONS`.

`domain.rs`:

```rust
/// The last crawl, for the sidebar. `stats` is the JSON written by the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CrawlRun {
    pub started_at: String,
    pub finished_at: Option<String>,
    pub stats: String,
}
```

`sqlite_store.rs` inherent methods:

```rust
pub struct ScoreUpdate {
    pub score: u8,
    pub target: Option<Target>,
    pub work_model: Option<WorkModel>,
    pub location: Option<String>,
    pub description: String,
    pub url_key: Option<String>,
}

pub struct UnscoredCard {
    pub id: String,
    pub source: String,
    pub title: String,
    pub company: String,
    pub job_url: Option<String>,
    pub description: String,
    pub location: Option<String>,
    pub stage: Stage,
    pub work_model: Option<WorkModel>,
}

/// `COALESCE` keeps stored values: a null update never erases data.
const APPLY_SCORE: &str = "UPDATE cards SET score = ?2, target = COALESCE(target, ?3), work_model = COALESCE(work_model, ?4), \
    location = COALESCE(location, ?5), description = CASE WHEN ?6 = '' THEN description ELSE ?6 END WHERE id = ?1";
```

- `apply_score(id, &ScoreUpdate)`, using `APPLY_SCORE`, `NotFound` when nothing changes.
- `unscored_cards()`: cards with `score IS NULL AND discard_reason IS NULL`. Discarded candidates with a description get a score too, so add `unscored_discarded()`, with the same shape, for `score IS NULL AND discard_reason IS NOT NULL AND description != ''`.
- `start_run(at) -> i64`, `finish_run(id, at, stats)`, `last_run() -> Option<CrawlRun>` (`ORDER BY id DESC LIMIT 1`).
- `known_keys() -> KnownKeys` (ids and URL keys from `cards` and from `dismissed_jobs`, as in `insert_new_jobs`; extract that query so both share it).

`insert_new_jobs` from `real-jobs-import` keeps serving new jobs; give `NewJob` an `Option<u8>` through its `card.score`. It already runs in one transaction; the pipeline calls it once per job so a closed app loses nothing.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | grep -E "last_run|applying|lists_only|test result: " | head -5`
Expected: PASS, three tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 8: Crawl pipeline, rescoring and the circuit breaker

**Requirement:** REQ-006, REQ-007, REQ-008
**Files:**
- Create: `src-tauri/src/crawler/pipeline.rs`
- Modify: `src-tauri/src/crawler/mod.rs`

**Depends on:** Tasks 3, 4, 6 and 7.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, consequences-reviewer, test-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/crawler/pipeline.rs` with the tests. A fake source returns preset jobs and detail outcomes:

```rust
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
        details: HashMap<String, DetailOutcome>,
        fetched: RefCell<Vec<String>>,
    }
    impl FakeSource {
        fn new(name: &'static str, jobs: Vec<Job>) -> Self {
            Self { name, jobs: Ok(jobs), details: HashMap::new(), fetched: RefCell::new(Vec::new()) }
        }
    }
    impl Source for FakeSource {
        fn name(&self) -> &'static str { self.name }
        fn owns(&self, source: &str) -> bool { source == self.name }
        fn collect(&self, _keep: &dyn Fn(&Job) -> bool, _progress: &dyn Fn(&str)) -> Result<Vec<Job>, String> { self.jobs.clone() }
        fn fetch_detail(&self, url: &str) -> DetailOutcome {
            self.fetched.borrow_mut().push(url.to_owned());
            self.details.get(url).cloned().unwrap_or(DetailOutcome::Failed("no page".into()))
        }
    }

    fn skills() -> BTreeSet<String> {
        ["python", "aws", "docker", "typescript", "react", "rust", "sql", "linux", "git", "fastapi"].map(str::to_owned).into_iter().collect()
    }
    fn setup() -> RunSettings {
        RunSettings { skills: skills(), min_score: 70, min_skills: 3, filters: Default::default(), now: crate::clock::fixed_now() }
    }
    fn job(title: &str, company: &str, url: &str, description: &str) -> Job {
        Job::new("gupy", "1", title, company, url).description(description).work_mode_hint(Some("remote"))
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
        let new = NewJob { card, url_key: crate::identity::url_key(url), description: description.into(), discard: None };
        store.insert_new_jobs(vec![new], &mut ImportReport::default()).unwrap();
    }

    #[test]
    fn routes_new_jobs_and_skips_known_ones() {
        let store = SqliteStore::open_in_memory().unwrap();
        let known = job("Conhecida Sênior", "Acme", "https://a.example/known", STRONG);
        imported(&store, &known.fingerprint(), "gupy", "https://a.example/known", STRONG);
        let source = FakeSource::new("gupy", vec![
            job("Engenheiro de IA Sênior", "Beta", "https://a.example/high", STRONG),
            job("Desenvolvedor Sênior", "Gama", "https://a.example/low", WEAK),
            job("Desenvolvedor Pleno", "Delta", "https://a.example/rej", STRONG),
            known,
            job("Outro título Sênior", "Acme", "https://www.a.example/known/", STRONG),
        ]);
        let sources: Vec<&dyn Source> = vec![&source];
        let stats = run_crawl(&store, &sources, &setup(), &|_| {});
        let inbox = store.list_cards().unwrap();
        let high = inbox.iter().find(|c| c.job_url.as_deref() == Some("https://a.example/high")).unwrap();
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
    fn a_deleted_job_never_returns_from_a_crawl() {
        let store = SqliteStore::open_in_memory().unwrap();
        let weak = job("Desenvolvedor Sênior", "Gama", "https://a.example/low", WEAK);
        let sources_one = FakeSource::new("gupy", vec![weak.clone()]);
        run_crawl(&store, &[&sources_one as &dyn Source], &setup(), &|_| {});
        store.delete_card(&weak.fingerprint(), "2026-11-02").unwrap();
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
        let good = FakeSource::new("gupy", vec![job("Engenheiro de IA Sênior", "Beta", "https://a.example/1", STRONG)]);
        let stats = run_crawl(&store, &[&broken as &dyn Source, &good], &setup(), &|_| {});
        assert_eq!(stats.inbox, 1);
        assert_eq!(stats.sources["solides"].error.as_deref(), Some("payload ausente"));
        assert_eq!(stats.sources["gupy"].error, None);
    }

    #[test]
    fn rescoring_keeps_user_edits_and_stage() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(&store, "aaaaaaaaaaaaaaaa", "gupy", "https://a.example/1", "");
        store.move_card("aaaaaaaaaaaaaaaa", Stage::Applied, None).unwrap();
        let details = crate::domain::CardDetails { notes: "nota".into(), ..Default::default() };
        store.update_card("aaaaaaaaaaaaaaaa", details).unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        source.details.insert("https://a.example/1".into(), DetailOutcome::Found(JobDetail {
            description: STRONG.into(), location: "Recife".into(), work_mode_hint: Some("remote".into()), published_at: None,
        }));
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let card = store.list_cards().unwrap().remove(0);
        assert!(card.score.is_some());
        assert_eq!((card.stage, card.notes.as_str()), (Stage::Applied, "nota"));
        assert_eq!(card.location.as_deref(), Some("Recife"));
    }

    #[test]
    fn a_stored_description_needs_no_request() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(&store, "aaaaaaaaaaaaaaaa", "gupy", "https://a.example/1", STRONG);
        let source = FakeSource::new("gupy", vec![]);
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert!(source.fetched.borrow().is_empty());
        assert!(store.list_cards().unwrap()[0].score.is_some());
    }

    #[test]
    fn a_low_rescore_stays_in_the_inbox() {
        let store = SqliteStore::open_in_memory().unwrap();
        imported(&store, "aaaaaaaaaaaaaaaa", "gupy", "https://a.example/1", "curto");
        let source = FakeSource::new("gupy", vec![]);
        run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let card = store.list_cards().unwrap().remove(0);
        assert!(card.score.unwrap() < 70);
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn closed_discards_and_failures_keep() {
        let store = SqliteStore::open_in_memory().unwrap();
        for (id, url) in [("aaaaaaaaaaaaaaaa", "https://a.example/closed"), ("bbbbbbbbbbbbbbbb", "https://a.example/fail"), ("cccccccccccccccc", "https://a.example/moved")] {
            imported(&store, id, "gupy", url, "");
        }
        imported(&store, "dddddddddddddddd", "lever", "https://a.example/unsupported", "");
        store.move_card("cccccccccccccccc", Stage::Applied, None).unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        source.details.insert("https://a.example/closed".into(), DetailOutcome::Closed);
        source.details.insert("https://a.example/moved".into(), DetailOutcome::Closed);
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        let discarded = store.list_discarded().unwrap();
        assert_eq!(discarded.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["aaaaaaaaaaaaaaaa"]);
        assert_eq!(discarded[0].reason, CLOSED_AD);
        let left: Vec<_> = store.list_cards().unwrap();
        assert_eq!(left.len(), 3);
        assert!(left.iter().all(|c| c.score.is_none()));
        assert_eq!(left.iter().find(|c| c.id == "cccccccccccccccc").unwrap().stage, Stage::Applied);
        assert_eq!(stats.rescore_closed, 1);
        assert!(!source.fetched.borrow().iter().any(|u| u.contains("unsupported")));
    }

    #[test]
    fn circuit_breaker_stops_a_mass_discard() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut source = FakeSource::new("gupy", vec![]);
        for i in 0..12 {
            let id = format!("{i:016x}");
            let url = format!("https://a.example/{i}");
            imported(&store, &id, "gupy", &url, "");
            source.details.insert(url, if i < 11 { DetailOutcome::Closed } else { DetailOutcome::Found(JobDetail { description: STRONG.into(), ..Default::default() }) });
        }
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert!(store.list_discarded().unwrap().is_empty());
        assert_eq!(stats.rescore_closed, 0);
        assert!(stats.sources["gupy"].error.as_deref().unwrap().contains("encerrad"));
        assert!(store.list_cards().unwrap().iter().any(|c| c.score.is_some()));
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
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::pipeline 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `run_crawl`"

**Step 3: Write minimal implementation**

```rust
//! One crawl: collect, filter, score, route, then rescore what has no score.
//! Each job commits on its own, so closing the app loses only the job in flight.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::clock::Now;
use crate::crawler::filters::{FilterSettings, count_skill_hits, passes_hard_filters};
use crate::crawler::job::Job;
use crate::crawler::scoring::score_job;
use crate::crawler::sources::{DetailOutcome, Source};
use crate::sqlite_store::SqliteStore;

pub const CLOSED_AD: &str = "anúncio encerrado";
/// Below this many pages one source's closed count proves nothing.
const BREAKER_MIN_PAGES: usize = 10;

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
```

`collect_new_jobs`, porting `collect.py`:
1. For each source, `collect` with `keep = passes_hard_filters(job, skills, 0, ...)`. On `Err`, record it in `stats.sources[name].error` and continue.
2. Group by fingerprint, choosing the first variant that passes (port `select_variant`, without `other_locations`, which the card does not store).
3. A rejected variant counts in `rejected_by_reason`, with the key `reason.split('(').next().trim()`, as Python does.
4. A passing job known by id or URL key (`store.known_keys()`, read once) counts in `known`; add each stored job's keys to the in-memory set so duplicates in one run also count.
5. Score it. Build a `NewJob` through a helper shared with `import.rs`: `import::job_to_card(&Job-like fields)`; extract `base_card` so both use it. Target comes from the title, work model from the hint or `detect_work_mode`, `found` event from `published_at` or today. Below `min_score` gets `discard: Some((BELOW_CUT, today))`.
6. Call `insert_new_jobs(vec![new], &mut report)` per job and map the report to the stats. A store error ends this source with an error, never a panic.

`rescore`, for `unscored_cards()` plus `unscored_discarded()`:
1. A card with a stored description is scored without a request, through `Job::new(...).description(...)`, for any source, including ones with no connector.
2. Else find the source with `owns(card.source)`; none leaves the card untouched and counts in `rescore_skipped`, a new `RunStats` field. No `job_url` counts as failed.
3. `fetch_detail` every page first and hold the outcomes per source; apply `Found` at once through `apply_score`, because it cannot do harm. Collect `Closed` per source.
4. After a source's pages: `if pages >= BREAKER_MIN_PAGES && closed * 2 > pages` then discard none and set `stats.sources[name].error` to `format!("{closed} de {pages} páginas parecem encerradas; nada foi descartado")`. Otherwise, each closed card in `Stage::Inbox` goes through `store.discard_card(id, CLOSED_AD, today)` and counts in `rescore_closed`; one in another stage stays, without a score.
5. `Failed` counts in `rescore_failed`.

`today` is `settings.now.format("%Y-%m-%d")`. Call `progress` with `"{source}: {done}/{total}"` per page.

`mock_data` is only visible to tests, which is all the tests here need.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::pipeline 2>&1 | grep -E "pipeline::tests|test result: " | head -14`
Expected: PASS, all nine tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 9: Run from the sidebar with real status

**Requirement:** REQ-009, REQ-010
**Files:**
- Create: `src-tauri/src/crawler/runner.rs`
- Create: `src/widgets/app-shell/CrawlerCard.tsx`
- Create: `src/widgets/app-shell/use-crawler.ts`
- Test: `src/widgets/app-shell/CrawlerCard.test.tsx`
- Modify: `src-tauri/src/crawler/mod.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/shared/api/board.ts`
- Modify: `src/widgets/app-shell/Sidebar.tsx`
- Modify: `src/pages/board/BoardPage.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/widgets/app-shell/AppShell.test.tsx`

**Depends on:** Task 8.

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails, security-reviewer

**Step 1: Write the failing test**

`src-tauri/src/crawler/runner.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config_error_code(&ConfigError::ProfileUnreadable), "profile_unreadable");
        assert_eq!(config_error_code(&ConfigError::NoSkills), "profile_no_skills");
        assert_eq!(config_error_code(&ConfigError::Invalid("min_score".into())), "config_invalid:min_score");
        assert_eq!(config_error_code(&ConfigError::Unreadable("x".into())), "config_unreadable");
    }
}
```

`src/widgets/app-shell/CrawlerCard.test.tsx`:

```tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { CrawlerCard } from './CrawlerCard';

const base = { running: false, progress: null, error: null, run: null, onRun: vi.fn() };
const finished = {
  startedAt: '2026-11-02T10:00:00-03:00',
  finishedAt: '2026-11-02T10:00:40-03:00',
  stats: JSON.stringify({ found: 50, inbox: 3, discarded: 4, sources: { gupy: { found: 30, error: null }, solides: { found: 20, error: 'x' } } }),
};

describe('CrawlerCard', () => {
  it('disables the button while running', () => {
    render(<CrawlerCard {...base} running progress="gupy: 3/10" />);
    expect(screen.getByRole('button', { name: 'Executando...' })).toBeDisabled();
    expect(screen.getByRole('status')).toHaveTextContent('gupy: 3/10');
  });

  it('runs on click', async () => {
    const onRun = vi.fn();
    render(<CrawlerCard {...base} onRun={onRun} />);
    await userEvent.click(screen.getByRole('button', { name: 'Executar agora' }));
    expect(onRun).toHaveBeenCalledOnce();
  });

  it('shows the real last run', () => {
    render(<CrawlerCard {...base} run={finished} />);
    expect(screen.getByText(/50 vagas/)).toBeInTheDocument();
    expect(screen.getByText(/40s/)).toBeInTheDocument();
    expect(screen.getByText(/1\/2 fontes ok/)).toBeInTheDocument();
    expect(screen.queryByText(/Próxima/)).not.toBeInTheDocument();
  });

  it('explains a never run and a refused run', () => {
    const { rerender } = render(<CrawlerCard {...base} />);
    expect(screen.getByText('O crawler ainda não rodou.')).toBeInTheDocument();
    rerender(<CrawlerCard {...base} error="config_missing" />);
    expect(screen.getByRole('alert')).toHaveTextContent('crawler.json');
    rerender(<CrawlerCard {...base} error="config_invalid:min_score" />);
    expect(screen.getByRole('alert')).toHaveTextContent('min_score');
  });

  it('marks an interrupted run', () => {
    render(<CrawlerCard {...base} run={{ ...finished, finishedAt: null }} />);
    expect(screen.getByText('A última execução foi interrompida.')).toBeInTheDocument();
  });

  it('survives stats it cannot read', () => {
    render(<CrawlerCard {...base} run={{ ...finished, stats: '{ nope' }} />);
    expect(screen.getByRole('button', { name: 'Executar agora' })).toBeEnabled();
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/widgets/app-shell/CrawlerCard --reporter=verbose 2>&1 | tail -4`
Expected: FAIL, "Failed to resolve import ./CrawlerCard"

**Step 3: Write minimal implementation**

`runner.rs`:

```rust
//! Starts a crawl on a worker thread. One at a time; progress goes out as events.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::crawler::config::ConfigError;

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
```

`AppState` gets `store: Arc<SqliteStore>`-backed access: change `store: Box<dyn Store>` to `store: Arc<dyn Store>` plus `crawl_store: Option<Arc<SqliteStore>>` and `guard: RunGuard`, `config_path: PathBuf`. Tests keep building it with `MockStore` and `crawl_store: None`.

Commands in `commands.rs`:
- `start_crawl(app, state) -> Result<(), String>`: `guard.try_start()`, then `load_crawl_setup` (error through `config_error_code`, with no request made), then `std::thread::spawn`. The worker builds `Client::new(UreqTransport::default())` and the connectors there, since they are not `Send`; it calls `store.start_run`, `run_crawl` with `progress = app.emit("crawler-progress", text)`, `store.finish_run` with `serde_json::to_string(&stats)`, then emits `crawler-finished`.
- `crawler_status(state) -> CrawlerStatus { running: bool, run: Option<CrawlRun> }`, with `CrawlerStatus` in `domain.rs` and a `#[ts(export)]`.

`lib.rs`: build `Arc<SqliteStore>`, register the commands, compute `config_path` as `data_dir/<dir>/crawler.json` through `db_path`'s directory logic, adding `db_path::app_dir(data_dir, debug)` and using it in `db_path`.

`src/shared/api/board.ts`:

```ts
export type CrawlerApi = {
  status(): Promise<CrawlerStatus>;
  start(): Promise<void>;
  onProgress(handler: (text: string) => void): Promise<() => void>;
  onFinished(handler: () => void): Promise<() => void>;
};
```

using `listen` from `@tauri-apps/api/event`. Wrap the two listeners so they call `handler(event.payload)`.

`use-crawler.ts` holds `{ running, progress, error, run }`, loads status on mount, subscribes to both events, unsubscribes on unmount, refreshes on finish, and exposes `start()`, which stores a rejected code in `error`. On finish it also dispatches `window.dispatchEvent(new Event('crawler-finished'))`; `BoardPage` reloads its cards on that event, adding a `reload` to `useBoard` that shares the first load's logic. On `crawler-progress` it reloads at most once per 5 s, so the board refreshes as cards change.

`CrawlerCard.tsx` is the existing `<section aria-label={t('crawler.title')}>` from `Sidebar.tsx`, moved here, driven by props: `{ running, progress, error, run, onRun }`. It parses `run.stats` in a `try/catch`, treating bad JSON as empty. Duration comes from the two timestamps. "Sources ok" is the entries with `error === null` over all entries. Keep the collapsed-sidebar classes (`group-data-[collapsible=icon]:...`) as they are now.

`Sidebar.tsx`: remove `CRAWLER_STATUS`, `notice` and the comment that says the crawler runs in the homelab; render `<CrawlerCard {...crawler} onRun={start} />` with `useCrawler`. Update `AppShell.test.tsx`, which asserted the old notice, to mock the API.

Locales: replace `crawler.lastRun`, `duration`, `sources`; remove `crawler.nextRun`. New keys (pt-BR / en): `crawler.running` "Executando..." / "Running..."; `crawler.never` "O crawler ainda não rodou." / "The crawler has not run yet."; `crawler.interrupted` "A última execução foi interrompida." / "The last run was interrupted."; `crawler.lastRun` "Última: {{time}} · {{total}} vagas · {{fresh}} novas" / "Last: {{time}} · {{total}} jobs · {{fresh}} new"; `crawler.duration` "Duração: {{duration}}s" / "Took {{duration}}s"; `crawler.sources` "{{ok}}/{{total}} fontes ok" / "{{ok}}/{{total}} sources ok"; `crawler.errors.config_missing` "Crie o crawler.json no diretório de dados do app, com o profile_path do seu currículo." / "Create crawler.json in the app data directory, with your resume's profile_path."; `crawler.errors.config_unreadable` "Não foi possível ler o crawler.json." / "crawler.json could not be read."; `crawler.errors.config_invalid` "Valor inválido no crawler.json: {{field}}." / "Invalid value in crawler.json: {{field}}."; `crawler.errors.profile_unreadable` "Não foi possível ler o currículo indicado em profile_path." / "The resume in profile_path could not be read."; `crawler.errors.profile_no_skills` "O currículo não tem a seção Stack técnica." / "The resume has no Stack técnica section."; `crawler.errors.already_running` "O crawler já está rodando." / "The crawler is already running."

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose 2>&1 | tail -5 && pnpm typecheck && cd src-tauri && cargo test 2>&1 | grep -E "runner::tests|test result: " | head -4`
Expected: PASS, the web suite and both runner tests green

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 10: Security docs, ROADMAP and the real crawl

**Requirement:** REQ-011
**Files:**
- Modify: `README.md`
- Modify: `README.pt-br.md`
- Modify: `ROADMAP.md`

**Team:**
- Track: docs
- Implementer: rpi-implementer (skills: readme-writer)
- Reviewers: self-review

**Step 1: Write the failing test**

```bash
grep -c "No network access\|Sem acesso à rede" README.md README.pt-br.md
```

**Step 2: Run test to verify it fails**
Run: `grep -c "crawler.json" README.md README.pt-br.md`
Expected: FAIL (the docs are stale): both counts `0`

**Step 3: Write minimal implementation**

`README.md`:
- Status: cards are saved locally and the crawler searches Gupy and Solides; AI integration is not there yet.
- Security: replace "No network access" with what is true now: **Requests only from Rust, only when you run the crawler.** It contacts `portal.gupy.io`, company `*.gupy.io` job pages, `vagas.solides.com.br`, and each host's `robots.txt`; it sends a User-Agent and the page path, and never your profile, notes or any credential. The webview's content security policy still allows only the IPC channel.
- Configuration section: `crawler.json` in `~/.local/share/dev.nandomoreira.jobradar/`, a minimal example with a fictional `profile_path`, the fields and their neutral defaults, and the profile's required `## Stack técnica` table.
- Usage row: "Find new jobs" | "Executar agora in the sidebar"; below-the-cut jobs land on Descartadas.

`README.pt-br.md`: the same, with full accents.

`ROADMAP.md`: milestone 4 status `Partial (Gupy and Solides; other sources in crawler-sources)`. The bar stays at 30%, because it only counts finished milestones.

Then run `check_readme.py` on both, `rpi-gates.sh tier1 --slug crawler-core .`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.

Manual check, recorded in `docs/specs/crawler-core/STATE.md`:
1. Without `crawler.json`, "Executar agora" shows the message and no request is made.
2. With a real `crawler.json` outside the repo, run it in the release data path or a `JOB_RADAR_DB` scratch copy of the imported database. New jobs appear; the sidebar shows progress and the real totals, surviving a restart.
3. Compare 5 scores with the Python crawler on the same text.
4. Sample at least 5 Gupy jobs marked closed by opening their links; a false "closed" is a High defect in `outcome_of`. Check the Solides 403 path leaves cards as "Sem nota".
5. Close the app during a crawl: reopen and see "interrompida"; the next run continues.

**Step 4: Run test to verify it passes**
Run: `grep -c "crawler.json" README.md README.pt-br.md; grep -c "No network access\|Sem acesso à rede" README.md README.pt-br.md`
Expected: PASS, the first counts at least `1` and the second counts `0`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
