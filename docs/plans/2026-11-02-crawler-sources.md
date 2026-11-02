# Plan: Crawler Sources

> **Spec:** docs/specs/crawler-sources/SPEC.md

**Goal:** Port the six remaining sources, near-duplicate detection and failure streaks, closing ROADMAP milestone 4.
**Architecture:** Each connector is one file behind the `Source` trait from `crawler-core`: pure parsing tested against Python vectors on recorded fixtures, plus fetching through the shared `Client`. Greenhouse, Lever and Ashby share a board loop and the Brazil relevance gate. Rescoring gains one single-job fetch per source that has one.
**Tech Stack:** Rust 2024, ureq 3, regex 1, sha1, serde_json, React 19, Vitest 5.
**Total Tasks:** 7
**Estimated Complexity:** medium
**Depends on:** `crawler-core` (`Source`, `Client`, `run_crawl`, `RunStats`, `crawler.json`, `scripts/gen_parity_vectors.py`). Implement it first.
**Lessons applied:** none

Conventions for every task:
- The reference is `../job-radar` (MIT). **Port behavior exactly.** When Rust and a vector disagree, Python is right.
- Rust commands run from `src-tauri/`. After each task: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- No `unwrap()`/`expect()` outside tests, except compiling a constant regex in `LazyLock`.
- Fixtures are recorded public pages copied from `../job-radar/tests/fixtures`, or fictional. No test touches the network. No real data from the server, no absolute home path.
- Every new connector follows `sources/gupy.rs`: `parse_*` pure, `outcome_of` for details, and a changed page shape counts as a source error, not zero jobs.

Batches: B1 = Tasks 1-2 (Brazil gate and boards), B2 = Tasks 3-5 (connectors and rescoring), B3 = Tasks 6-7 (near-duplicates, streaks, docs).

---

## Checks

**Run rust**: `cd src-tauri && cargo test 2>&1`
**Run web**: `pnpm vitest run --reporter=verbose`

- C1 (REQ-001): the Brazil rule equals Python's on every vector. Proof: run rust | crawler::geo::tests::matches_the_python_vectors ... ok
- C2 (REQ-001): Greenhouse parsing and gate equal Python's. Proof: run rust | crawler::sources::greenhouse::tests::parses_the_recorded_board_like_python ... ok
- C3 (REQ-001): Lever parsing and gate equal Python's. Proof: run rust | crawler::sources::lever::tests::parses_the_recorded_board_like_python ... ok
- C4 (REQ-001): Ashby parsing and gate equal Python's. Proof: run rust | crawler::sources::ashby::tests::parses_the_recorded_board_like_python ... ok
- C5 (REQ-002): `ats_boards` replaces the defaults and reports an invalid entry. Proof: run rust | crawler::boards::tests::config_replaces_the_defaults_and_reports_invalid_entries ... ok
- C6 (REQ-003): InHire parsing equals Python's and details are fetched only for kept jobs. Proof: run rust | crawler::sources::inhire::tests::parses_the_recorded_pages_like_python ... ok
- C7 (REQ-003): GitHub issues parse like Python's. Proof: run rust | crawler::sources::github::tests::parses_issues_like_python ... ok
- C8 (REQ-003): the token goes only to the GitHub API. Proof: run rust | crawler::sources::github::tests::sends_the_token_only_to_the_github_api ... ok
- C9 (REQ-003): ProgramaThor parses like Python's. Proof: run rust | crawler::sources::programathor::tests::parses_the_listing_like_python ... ok
- C10 (REQ-004): each source's single-job fetch follows the closed rules, and a source without one never discards. Proof: run rust | crawler::sources::tests::single_job_fetch_follows_the_closed_rules ... ok
- C11 (REQ-005): simhash equals Python's and near-duplicates follow the same rule. Proof: run rust | crawler::identity::tests::simhash_matches_the_python_vectors ... ok
- C12 (REQ-005): a near-duplicate is dropped, in a run and against a stored job. Proof: run rust | crawler::pipeline::tests::near_duplicates_are_dropped_and_counted ... ok
- C13 (REQ-006): the failure streak grows, resets and survives a reopen. Proof: run rust | sqlite_store::tests::failure_streaks_grow_reset_and_persist ... ok
- C14 (REQ-006): the sidebar names a source failing on consecutive runs. Proof: run web | ✓ CrawlerCard names a source failing on consecutive runs

---

### Task 1: Brazil relevance gate

**Requirement:** REQ-001
**Files:**
- Create: `src-tauri/src/crawler/geo.rs`
- Create: `src-tauri/tests/fixtures/parity/geo.json`
- From `crawler-core`, modified here: `scripts/gen_parity_vectors.py`
- From `crawler-core`, modified here: `src-tauri/src/crawler/mod.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Add to `gen_parity_vectors.py`, and write `geo.json` from `main()`:

```python
from job_radar.geo import is_brazil_relevant, normalize_location

def geo_vectors():
    locations = ["São Paulo, SP", "Remote - BR", "Remote", "Remote - LATAM", "Mexico City", "Buenos Aires, Argentina",
                 "Recife", "Maranhão", "MA, USA", "Boston, MA", "Goiânia - GO", "Brazil", "Brasil", "New York", "", "Europe", "São Paulo, Brasil / Remote", "Curitiba/PR"]
    countries = ["", "BR", "Brazil", "US", "Mexico", "Argentina"]
    modes = [None, "remote", "hybrid", "onsite"]
    return {"relevant": [[l, c, m, is_brazil_relevant(location=l, country=c, work_mode=m)] for l in locations for c in countries for m in modes],
            "normalized": [[l, normalize_location(l)] for l in locations]}
```

`src-tauri/src/crawler/geo.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn matches_the_python_vectors() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/geo.json")).unwrap();
        let rows = v["relevant"].as_array().unwrap();
        assert!(rows.len() > 100);
        assert!(rows.iter().any(|r| r[3].as_bool().unwrap()) && rows.iter().any(|r| !r[3].as_bool().unwrap()));
        for row in rows {
            assert_eq!(is_brazil_relevant(row[0].as_str().unwrap(), row[1].as_str().unwrap(), row[2].as_str()), row[3].as_bool().unwrap(), "{row}");
        }
        for row in v["normalized"].as_array().unwrap() {
            assert_eq!(normalize_location(row[0].as_str().unwrap()), row[1].as_str().unwrap(), "{row}");
        }
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::geo 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `is_brazil_relevant`"

**Step 3: Write minimal implementation**

Port `../job-radar/src/job_radar/geo.py` line for line (304 lines): the state and city tables verbatim, `normalize_location`, `_mentions_foreign_country`, `_uf_counts_as_brazil`, `_mentions_brazil`, `is_brazil_relevant(location: &str, country: &str, work_mode: Option<&str>) -> bool`. Keep the comment that a foreign country written in full beats a two-letter state code (MA is not Maranhão). Use `regex` for the patterns; add `fancy_regex` only where one uses lookaround.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::geo 2>&1 | grep -E "geo::tests|test result: " | head -2`
Expected: PASS, `crawler::geo::tests::matches_the_python_vectors ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Boards and the ATS connectors

**Requirement:** REQ-001, REQ-002
**Files:**
- Create: `src-tauri/src/crawler/boards.rs`
- Create: `src-tauri/src/crawler/sources/ats.rs`
- Create: `src-tauri/src/crawler/sources/greenhouse.rs`
- Create: `src-tauri/src/crawler/sources/lever.rs`
- Create: `src-tauri/src/crawler/sources/ashby.rs`
- Create: `src-tauri/tests/fixtures/greenhouse_board.json`
- Create: `src-tauri/tests/fixtures/lever_postings.json`
- Create: `src-tauri/tests/fixtures/ashby_board.json`
- From `crawler-core`, modified here: `scripts/gen_parity_vectors.py`
- From `crawler-core`, modified here: `src-tauri/tests/fixtures/parity/sources.json`
- From `crawler-core`, modified here: `src-tauri/src/crawler/config.rs`
- From `crawler-core`, modified here: `src-tauri/src/crawler/mod.rs`
- From `crawler-core`, modified here: `src-tauri/src/crawler/sources/mod.rs`

**Depends on:** Task 1.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Copy the three fixtures from `../job-radar/tests/fixtures`. In `gen_parity_vectors.py`'s `source_vectors()`, add one key per connector, each holding the jobs that **pass the gate**, using Python's `_parse_board`, so the Rust test covers parse and gate together:

```python
from job_radar.sources import ashby as py_ashby, greenhouse as py_greenhouse, lever as py_lever
from job_radar.boards import parse_ats_boards, resolve_boards

def board_rows(module, payload, slug, company):
    # Same gate as _ats.fetch_ats_boards; read that function to keep the call exact.
    from job_radar.geo import is_brazil_relevant
    rows = []
    for job, location, country in module._parse_board(payload, slug, company):
        if is_brazil_relevant(location=location, country=country, work_mode=job.work_mode_hint):
            rows.append(job_dict(job))
    return rows
```

Read `sources/_ats.py` and each `_parse_board` first; `GateRow`'s exact shape decides this tuple. Add `"boards_config"`: the result of `parse_ats_boards` on `"greenhouse:a=Nome A,lever:b,ashby:c=C,inhire:d=D,nope:x,greenhouse:"`, and `"boards_default": resolve_boards({})`.

Each connector file gets:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::assert_jobs_match;

    #[test]
    fn parses_the_recorded_board_like_python() {
        let payload = include_str!("../../../tests/fixtures/greenhouse_board.json");
        let jobs = gated_jobs(payload, "acme", "Acme").unwrap();
        assert_jobs_match(&jobs, "greenhouse");
        assert!(!jobs.is_empty());
        assert!(gated_jobs("{ broken", "acme", "Acme").is_err());
    }
}
```

The same for `lever` and `ashby`, with their fixture and vector key.

`src-tauri/src/crawler/boards.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn config_replaces_the_defaults_and_reports_invalid_entries() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/sources.json")).unwrap();
        let (boards, invalid) = parse_boards("greenhouse:a=Nome A,lever:b,ashby:c=C,inhire:d=D,nope:x,greenhouse:");
        assert_eq!(serde_json::to_value(&boards).unwrap(), v["boards_config"]);
        assert_eq!(invalid, ["nope:x", "greenhouse:"]);
        assert_eq!(serde_json::to_value(default_boards()).unwrap(), v["boards_default"]);
        assert_eq!(resolve_boards(None).0, default_boards());
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::boards 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `parse_boards`"

**Step 3: Write minimal implementation**

- `boards.rs`: ports `boards.py`, returning `BTreeMap<String, BTreeMap<String, String>>` so the JSON comparison is stable, plus the invalid entries, which Python drops silently and which the spec wants reported. Add `ats_boards: Option<String>` to `RawConfig` and `CrawlerConfig`; invalid entries go to `RunStats` as `invalid_boards`.
- `sources/ats.rs`: ports `_ats.fetch_ats_boards`, the per-board loop. A 404 on one board records an error and continues with the others.
- `greenhouse.rs`, `lever.rs`, `ashby.rs`: each ports its Python file, with `gated_jobs(payload, slug, company) -> Result<Vec<Job>, String>`, the `API` URL, `owns(source)`, and `impl Source`. A payload with an unexpected top-level shape is `Err`.
- Register them in the sources list that `start_crawl` builds, with the boards from config or defaults.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler 2>&1 | grep -E "boards::tests|greenhouse|lever|ashby|test result: " | head -6`
Expected: PASS, every listed test `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: InHire

**Requirement:** REQ-003
**Files:**
- Create: `src-tauri/src/crawler/sources/inhire.rs`
- Create: `src-tauri/tests/fixtures/inhire_page.json`
- Create: `src-tauri/tests/fixtures/inhire_job.json`
- From `crawler-core`, modified here: `scripts/gen_parity_vectors.py`
- From `crawler-core`, modified here: `src-tauri/tests/fixtures/parity/sources.json`
- From `crawler-core`, modified here: `src-tauri/src/crawler/sources/mod.rs`

**Depends on:** Task 2.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Copy both fixtures. Vectors: `"inhire"`, Python's `parse_listing` over each page item followed by `apply_detail` with `inhire_job.json` for the item whose id matches (read `tests/test_inhire.py` for the call).

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::assert_jobs_match;

    #[test]
    fn parses_the_recorded_pages_like_python() {
        let page = include_str!("../../../tests/fixtures/inhire_page.json");
        let detail = include_str!("../../../tests/fixtures/inhire_job.json");
        let jobs = jobs_with_details(page, "acme", "Acme", &|_| Some(detail.to_owned()), &|_| true).unwrap();
        assert_jobs_match(&jobs, "inhire");
    }

    #[test]
    fn fetches_details_only_for_kept_jobs() {
        let page = include_str!("../../../tests/fixtures/inhire_page.json");
        let calls = std::cell::Cell::new(0);
        let fetch = |_: &str| {
            calls.set(calls.get() + 1);
            None
        };
        jobs_with_details(page, "acme", "Acme", &fetch, &|_| false).unwrap();
        assert_eq!(calls.get(), 0);
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::sources::inhire 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `jobs_with_details`"

**Step 3: Write minimal implementation**

Port `sources/inhire.py`, including the per-tenant headers, 1 s between details, `country_from_location` with the Brazil gate, and `apply_detail`. `jobs_with_details(page, slug, company, fetch_detail, keep)` is the pure core that `collect` wraps with real requests. Assert the vectors with `assert_jobs_match`, whose `published_at` coverage comes from the fingerprint and fields already listed there.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::sources::inhire 2>&1 | grep -E "inhire::tests|test result: " | head -3`
Expected: PASS, both tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: GitHub issues and ProgramaThor

**Requirement:** REQ-003
**Files:**
- Create: `src-tauri/src/crawler/sources/github.rs`
- Create: `src-tauri/src/crawler/sources/programathor.rs`
- Create: `src-tauri/tests/fixtures/github_issues.json`
- Create: `src-tauri/tests/fixtures/programathor_listing.html`
- From `crawler-core`, modified here: `scripts/gen_parity_vectors.py`
- From `crawler-core`, modified here: `src-tauri/tests/fixtures/parity/sources.json`
- From `crawler-core`, modified here: `src-tauri/src/crawler/config.rs`
- From `crawler-core`, modified here: `src-tauri/src/crawler/http.rs`
- From `crawler-core`, modified here: `src-tauri/src/crawler/sources/mod.rs`

**Depends on:** Task 2.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Write both fixtures by hand, fictional, shaped after `../job-radar/tests/test_github_repos.py` and `tests/test_programathor.py`, covering: bracket titles, `na Empresa`, a trailing `(Empresa)`, emoji, labels with modalities; on ProgramaThor, a `NOVA` badge, an expired card (`Vencida`), tags and icons. Vectors: `"github"`, Python's `parse_issue(issue, "backend-br/vagas")` per item; `"programathor"`, `parse_listing(html)`; `"github_titles"`, `parse_title` over the bracket cases.

`github.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::{assert_jobs_match, vectors};

    #[test]
    fn parses_issues_like_python() {
        let issues: Vec<serde_json::Value> = serde_json::from_str(include_str!("../../../tests/fixtures/github_issues.json")).unwrap();
        let jobs: Vec<_> = issues.iter().map(|issue| parse_issue(issue, "backend-br/vagas")).collect();
        assert_jobs_match(&jobs, "github");
        for row in vectors()["github_titles"].as_array().unwrap() {
            let parsed = parse_title(row[0].as_str().unwrap());
            assert_eq!(serde_json::to_value(parsed).unwrap(), row[1], "{row}");
        }
    }

    #[test]
    fn sends_the_token_only_to_the_github_api() {
        assert_eq!(auth_header("https://api.github.com/repos/a/b/issues", Some("t")), Some("Bearer t".to_owned()));
        assert_eq!(auth_header("https://example.com/x", Some("t")), None);
        assert_eq!(auth_header("https://api.github.com.evil.example/x", Some("t")), None);
        assert_eq!(auth_header("https://api.github.com/x", None), None);
        assert_eq!(auth_header("https://api.github.com/x", Some("")), None);
    }

    #[test]
    fn a_debug_print_never_shows_the_token() {
        let source = GitHubSource::for_tests(Some("segredo".into()));
        assert!(!format!("{source:?}").contains("segredo"));
    }
}
```

`programathor.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::sources::tests_support::assert_jobs_match;

    #[test]
    fn parses_the_listing_like_python() {
        let jobs = parse_listing(include_str!("../../../tests/fixtures/programathor_listing.html")).unwrap();
        assert_jobs_match(&jobs, "programathor");
        assert!(!jobs.is_empty());
        assert!(parse_listing("<html>sem cards</html>").is_err());
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test crawler::sources::github crawler::sources::programathor 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `parse_issue`"

**Step 3: Write minimal implementation**

Port `sources/github_repos.py` and `sources/programathor.py`. `GetOptions` gains `authorization: Option<String>`, and `Transport::get` takes it as a parameter. Update the fakes in `http.rs`'s tests. `auth_header` compares the parsed host to exactly `api.github.com`:

```rust
pub fn auth_header(url: &str, token: Option<&str>) -> Option<String> {
    let token = token.filter(|t| !t.is_empty())?;
    let host = url::Url::parse(url).ok()?.host_str()?.to_owned();
    (host == "api.github.com").then(|| format!("Bearer {token}"))
}
```

`GitHubSource` implements `Debug` by hand, printing `token: <redacted>`. Add `github_token: Option<String>` to `RawConfig`; never put it in `RunStats` or send it over IPC. ProgramaThor: an HTTP 200 with zero cards is an error for this page's parse, as the Gupy connector does. A rate limit (403 or 429 from GitHub) is a source error, not zero jobs. Register both.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test crawler::sources 2>&1 | grep -E "github::tests|programathor::tests|test result: " | head -6`
Expected: PASS, the four tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: Rescoring for these sources

**Requirement:** REQ-004
**Files:**
- From `crawler-core`, modified here: `src-tauri/src/crawler/sources/mod.rs`
- Modify: `src-tauri/src/crawler/sources/greenhouse.rs`
- Modify: `src-tauri/src/crawler/sources/lever.rs`
- Modify: `src-tauri/src/crawler/sources/ashby.rs`
- Modify: `src-tauri/src/crawler/sources/inhire.rs`
- Modify: `src-tauri/src/crawler/sources/github.rs`
- Modify: `src-tauri/src/crawler/sources/programathor.rs`

**Depends on:** Tasks 2, 3 and 4.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, consequences-reviewer, test-reviewer

**Step 1: Write the failing test**

Add to `sources/mod.rs` tests:

```rust
    #[test]
    fn single_job_fetch_follows_the_closed_rules() {
        use crate::crawler::http::HttpError;
        assert_eq!(greenhouse::detail_url("https://boards.greenhouse.io/acme/jobs/123?gh_jid=123").as_deref(), Some("https://boards-api.greenhouse.io/v1/boards/acme/jobs/123"));
        assert_eq!(greenhouse::detail_url("https://acme.com/careers/123"), None);
        assert_eq!(lever::detail_url("https://jobs.lever.co/acme/abc-def").as_deref(), Some("https://api.lever.co/v0/postings/acme/abc-def?mode=json"));
        assert_eq!(github::detail_url("https://github.com/backend-br/vagas/issues/9").as_deref(), Some("https://api.github.com/repos/backend-br/vagas/issues/9"));
        for outcome in [greenhouse::outcome_of(Err(HttpError::Status(404))), lever::outcome_of(Err(HttpError::Status(410))), github::outcome_of(Err(HttpError::Status(404)))] {
            assert_eq!(outcome, DetailOutcome::Closed);
        }
        for outcome in [greenhouse::outcome_of(Err(HttpError::Blocked)), lever::outcome_of(Err(HttpError::Status(500))), github::outcome_of(Err(HttpError::Status(403)))] {
            assert!(matches!(outcome, DetailOutcome::Failed(_)));
        }
        // A closed GitHub issue is a closed ad; an open one has a body.
        assert_eq!(github::outcome_of(Ok(br#"{"state":"closed","body":"x"}"#.to_vec())), DetailOutcome::Closed);
        assert!(matches!(github::outcome_of(Ok(br#"{"state":"open","body":"Python"}"#.to_vec())), DetailOutcome::Found(_)));
        // A source with no way to fetch one job never closes anything.
        assert!(matches!(ashby::outcome_for_url("https://jobs.ashbyhq.com/acme/1"), DetailOutcome::Failed(_) | DetailOutcome::Found(_)));
        assert!(matches!(programathor::outcome_for_url_without_fetch("https://programathor.com.br/jobs/1"), DetailOutcome::Failed(_)));
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test single_job_fetch 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `detail_url`"

**Step 3: Write minimal implementation**

For each source, `fetch_detail(url)`:
- **Greenhouse:** `detail_url` turns `boards.greenhouse.io/{slug}/jobs/{id}` and `job-boards.greenhouse.io/{slug}/jobs/{id}` into the API; other hosts give `Failed`. Parse `content` as in `parse_job`.
- **Lever:** `jobs.lever.co/{slug}/{id}` to the API with `mode=json`.
- **Ashby:** has no single-job endpoint in the Python code. Check whether the job page embeds JSON-LD with a live fetch before coding; if not, `fetch_detail` returns `Failed("no single-job fetch")`, which the spec accepts. Adjust the test's `outcome_for_url` line to the choice and record it in `STATE.md`.
- **InHire:** reuse the detail call already ported; 404 or 410 close.
- **GitHub:** `github.com/{owner}/{repo}/issues/{n}` to the API with the token; `state: closed` closes.
- **ProgramaThor:** fetch the page and read the description if the listing's job page has a stable block; otherwise `Failed`. Decide on a live fetch and record it in `STATE.md`. `Vencida` on the page closes it, matching `_EXPIRED_RE`.

Only `Status(404|410)` and each source's explicit closed marker return `Closed`. The circuit breaker in `crawler-core` already covers every source.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test single_job_fetch 2>&1 | grep -E "single_job|test result: " | head -2`
Expected: PASS, `crawler::sources::tests::single_job_fetch_follows_the_closed_rules ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: Near-duplicates and failure streaks

**Requirement:** REQ-005, REQ-006
**Files:**
- Create: `src-tauri/migrations/004_simhash_streaks.sql`
- Create: `src-tauri/tests/fixtures/parity/simhash.json`
- From `crawler-core`, modified here: `scripts/gen_parity_vectors.py`
- Modify: `src-tauri/Cargo.toml`
- From `crawler-core`, modified here: `src-tauri/src/crawler/identity.rs`
- From `crawler-core`, modified here: `src-tauri/src/crawler/pipeline.rs`
- Modify: `src-tauri/src/sqlite_store.rs`
- From `crawler-core`, modified here: `src/widgets/app-shell/CrawlerCard.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- From `crawler-core`, test extended here: `src/widgets/app-shell/CrawlerCard.test.tsx`

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Vectors in `gen_parity_vectors.py`:

```python
from job_radar.identity import hamming, simhash, simhash_hex

def simhash_vectors():
    long_a = " ".join(f"palavra{i}" for i in range(80))
    long_b = long_a.replace("palavra40", "outra")
    texts = ["", "curto", "x " * 99, long_a, long_b, "Coração " * 60]
    values = [simhash(t) for t in texts]
    return {"texts": [[t, simhash_hex(v)] for t, v in zip(texts, values)],
            "hamming": hamming(values[3], values[4])}
```

`crawler/identity.rs` tests:

```rust
    #[test]
    fn simhash_matches_the_python_vectors() {
        let v: Value = serde_json::from_str(include_str!("../../tests/fixtures/parity/simhash.json")).unwrap();
        for row in v["texts"].as_array().unwrap() {
            assert_eq!(simhash_hex(simhash(row[0].as_str().unwrap())), row[1].as_str().unwrap(), "{row}");
        }
        let texts = v["texts"].as_array().unwrap();
        let a = simhash(texts[3][0].as_str().unwrap()).unwrap();
        let b = simhash(texts[4][0].as_str().unwrap()).unwrap();
        assert_eq!(u64::from((a ^ b).count_ones()), v["hamming"].as_u64().unwrap());
    }
```

`crawler/pipeline.rs` tests:

```rust
    #[test]
    fn near_duplicates_are_dropped_and_counted() {
        let body: String = (0..80).map(|i| format!("palavra{i} ")).collect::<String>() + STRONG;
        let store = SqliteStore::open_in_memory().unwrap();
        let first = job("Engenheiro de IA Sênior", "Cliente", "https://a.example/1", &body);
        let copy = job("Engenheira de Dados Sênior", "Consultoria", "https://a.example/2", &body);
        let same_company = job("Data Engineer Sênior", "Cliente", "https://a.example/3", &body);
        let source = FakeSource::new("gupy", vec![first, copy, same_company]);
        let stats = run_crawl(&store, &[&source as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.near_duplicates, 1);
        assert_eq!(store.list_cards().unwrap().len(), 2);

        // A later run compares with what is stored.
        let later = FakeSource::new("gupy", vec![job("Tech Lead Sênior", "Outra Consultoria", "https://a.example/4", &body)]);
        let stats = run_crawl(&store, &[&later as &dyn Source], &setup(), &|_| {});
        assert_eq!(stats.near_duplicates, 1);
    }
```

`sqlite_store.rs` tests:

```rust
    #[test]
    fn failure_streaks_grow_reset_and_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            assert_eq!(store.record_source_result("gupy", true).unwrap(), 1);
            assert_eq!(store.record_source_result("gupy", true).unwrap(), 2);
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.record_source_result("gupy", true).unwrap(), 3);
        assert_eq!(store.record_source_result("gupy", false).unwrap(), 0);
        assert_eq!(store.record_source_result("gupy", true).unwrap(), 1);
    }
```

`CrawlerCard.test.tsx`:

```tsx
  it('names a source failing on consecutive runs', () => {
    const stats = JSON.stringify({ found: 1, sources: { gupy: { found: 1, error: null, streak: 0 }, solides: { found: 0, error: 'x', streak: 3 }, lever: { found: 0, error: 'y', streak: 1 } } });
    render(<CrawlerCard {...base} run={{ ...finished, stats }} />);
    expect(screen.getByText('solides falhou nas últimas 3 execuções')).toBeInTheDocument();
    expect(screen.queryByText(/lever falhou/)).not.toBeInTheDocument();
  });
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test simhash failure_streaks 2>&1 | tail -4`
Expected: FAIL to compile with "cannot find function `simhash`"

**Step 3: Write minimal implementation**

- `004_simhash_streaks.sql`: `ALTER TABLE cards ADD COLUMN simhash TEXT;` and `CREATE TABLE source_failures (source TEXT PRIMARY KEY, streak INTEGER NOT NULL);`.
- `Cargo.toml`: `sha1 = "0.10"`.
- `crawler/identity.rs`: port `simhash` (64 bits over 3-word shingles of `fold`, SHA-1, under 200 folded chars gives `None`), `simhash_hex`, and `are_near_duplicates`, keeping the MIT credit to career-ops in the comment, as `identity.py` does.
- `pipeline.rs`: port `drop_near_duplicates` and `matches_history_simhash`, computed once per job. Store the hex in `cards.simhash` on insert and in `apply_score`. `RunStats` gains `near_duplicates`. Read the stored `(simhash, company)` once per run; a stored job from the same company is not a near-duplicate. A stored job without a simhash never matches. Near-duplicates are skipped like known jobs: not stored, not on Discarded.
- `sqlite_store.rs`: `record_source_result(source, failed) -> i64`. `SourceStats` gains `streak`, filled once per source at the end of the run.
- `CrawlerCard.tsx`: a source with `streak >= 2` shows `crawler.failing`, "{{source}} falhou nas últimas {{count}} execuções" / "{{source}} failed the last {{count}} runs".

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "simhash|near_dup|failure_streaks|test result: " | head -5; cd .. && pnpm vitest run src/widgets/app-shell --reporter=verbose 2>&1 | tail -4`
Expected: PASS, all of them green

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: Docs, ROADMAP and the full crawl

**Requirement:** REQ-007
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
grep -c "api.github.com" README.md README.pt-br.md
```

**Step 2: Run test to verify it fails**
Run: `grep -c "api.github.com" README.md README.pt-br.md`
Expected: FAIL (the docs are stale): both counts `0`

**Step 3: Write minimal implementation**

`README.md`:
- Security: list every host now contacted: `boards-api.greenhouse.io`, `api.lever.co`, `jobs.lever.co`, `api.ashbyhq.com`, `api.inhire.app`, `*.inhire.app`, `api.github.com`, `programathor.com.br`, in addition to Gupy and Solides. `github_token` is optional, sent only to `api.github.com`, never logged; keep `crawler.json` private.
- Configuration: `ats_boards` (format and an example with fictional slugs) and `github_token`.
- Status: all eight sources.

`README.pt-br.md`: the same, with full accents.

`ROADMAP.md`: milestone 4 `Done (independent review and manual check pending)`, the bar at `█████████░░░░░░░░░░░  45%  (4 of 11 milestones, weighted by effort)`, and the total at `**45 done**`.

Then run `check_readme.py` on both, `./node_modules/.bin/prettier --write ROADMAP.md`, `rpi-gates.sh tier1 --slug crawler-sources .`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.

Manual check, recorded in `docs/specs/crawler-sources/STATE.md`:
1. A crawl returns jobs per source; compare the totals with the Python crawler's `last_run.json` on the same day, by source.
2. The 34 imported jobs from these sources get a score, or stay "Sem nota" with a stated reason.
3. Remove `github_token` and trigger a rate limit, or point `ats_boards` to a missing board: the source shows an error and the others finish.

**Step 4: Run test to verify it passes**
Run: `grep -c "api.github.com" README.md README.pt-br.md; grep -c "45%" ROADMAP.md`
Expected: PASS, all counts at least `1`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
