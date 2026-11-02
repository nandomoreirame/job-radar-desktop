# Spec: Crawler Sources

## Problem Statement

After `crawler-core` the app only searches Gupy and Solides, while the Python crawler also covers
six more sources, and the 34 imported jobs from those sources still have no score. Greenhouse alone
holds 73 of the last 137 candidates, so most of the good ATS jobs would go missing.

## Goals

- The app covers all eight sources of the Python crawler.
- Every imported job can get a score from its page.
- A job posted on two sources under different names does not appear twice.
- A source that keeps failing is visible in the sidebar.

## Context

Depends on `crawler-core` (HTTP client, identity, filters, pipeline, rescoring). Reference: the MIT
repo `nandomoreirame/job-radar`, `src/job_radar/sources/`, `boards.py`, `geo.py`, `dedup.py`,
`health.py`. With this slice the ROADMAP milestone 4 closes.

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| Connectors to port: greenhouse (158 lines), lever (115), ashby (138), inhire (173), github_repos (159), programathor (110), plus `_ats.py` (57) | `wc -l` on 2026-11-02 | yes |
| Boards and their config format live in `boards.py` (90 lines); Brazil relevance in `geo.py` (304) | `job_radar/boards.py`, `job_radar/geo.py` | yes |
| Near-duplicates use a 64-bit simhash, hamming at most 5, different company keys, text of at least 200 chars | `job_radar/identity.py:128-129,302-345` | yes |
| The imported jobs not covered by `crawler-core` are greenhouse 15, github 14, inhire 3, ashby 2 | `seen.db` on the server, 2026-11-02 | yes |
| Fixtures exist for greenhouse, lever, ashby and inhire; none listed for github or programathor | `tests/fixtures/` | yes |
| Failure streaks per source are kept in `health.py` (133 lines) | `job_radar/health.py` | yes |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN the Greenhouse, Lever and Ashby connectors parse a recorded board THEN they SHALL produce the same jobs as the Python connectors, and jobs not relevant to Brazil SHALL be dropped by the same rule, proved by vectors generated from the Python code.
2. **[REQ-002]** WHEN `crawler.json` sets `ats_boards` THEN those boards SHALL replace the defaults, in the same format as `ATS_BOARDS`, and an invalid entry SHALL be ignored and reported.
3. **[REQ-003]** WHEN the InHire, GitHub and ProgramaThor connectors parse a recorded page THEN they SHALL produce the same jobs as the Python connectors; InHire SHALL fetch details only for jobs passing the cheap filters; an optional `github_token` in `crawler.json` SHALL be sent only to the GitHub API.
4. **[REQ-004]** WHEN rescoring meets a job from a source of this slice THEN it SHALL use that source's page or API, with the closed rules of `crawler-core`; a source with no way to fetch one job SHALL leave it without a score, never discard it.
5. **[REQ-005]** WHEN two jobs in a run, or a new job and a stored one, are near-duplicates by simhash with different companies THEN only the first SHALL be kept and the other SHALL count in the statistics.
6. **[REQ-006]** WHEN a source fails on consecutive runs THEN the sidebar SHALL show which one and for how many runs, with the streak persisted.

### P2 (Next)

1. **[REQ-007]** WHEN this ships THEN README (en, pt-BR) SHALL list every contacted host and `ats_boards`/`github_token`, and ROADMAP SHALL mark milestone 4 done with the bar updated.

### Out of Scope

- New sources beyond the eight of the Python crawler; Gmail alerts.
- Scheduled runs and a settings UI.

## Edge Cases

- **GitHub rate limit without a token** - a source error, not zero jobs.
- **A board that no longer exists (404)** - that board is an error, the others continue.
- **Short description under 200 chars** - no simhash, so no near-duplicate match, as in Python.
- **Card imported with a stored simhash** - none is imported; stored jobs get one when they gain a description.
- **`github_token` empty** - requests go unauthenticated.

## Data Model

Migration 4 adds `simhash TEXT` to `cards` and `source_failures(source TEXT PRIMARY KEY, streak INTEGER NOT NULL)`.

## Architecture Decisions

1. **One connector per file behind a small trait** - Same split as Python: pure `parse_*`, tested on fixtures, and a fetch function; adding a source touches one file and a registry.
2. **Parity by generated vectors again** - Same mechanism as `crawler-core`; GitHub and ProgramaThor get fictional fixtures here since Python has none recorded.
3. **Default boards in code, overridable by config** - Same defaults as `boards.py`, none of them personal.

## Security

- New hosts: the three ATS APIs, InHire, GitHub API, ProgramaThor, all through the `crawler-core` client.
- `github_token` is read from `crawler.json` (mode 0600 expected), sent only to `api.github.com`, never logged, never returned over IPC.
- Fixtures contain public or fictional data only.

## Success Criteria

- A crawl returns jobs from all eight sources with results equal to the Python crawler's on the vectors.
- After a run, no imported job lacks a score except by a transient failure or a source with no single-job fetch.
- ROADMAP milestone 4 is done.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Geo rules drift from Python | Medium | Vectors over each branch of `geo.py` |
| GitHub issue-based jobs have no single-page structure | Medium | They keep no score rather than a guess (REQ-004) |
| More sources slow a run | Low | Per-host pacing keeps sources independent; progress shows in the sidebar |

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | Greenhouse, Lever, Ashby and Brazil rule | P1 | Pending |
| REQ-002 | `ats_boards` config | P1 | Pending |
| REQ-003 | InHire, GitHub, ProgramaThor | P1 | Pending |
| REQ-004 | Rescoring for these sources | P1 | Pending |
| REQ-005 | Near-duplicates by simhash | P1 | Pending |
| REQ-006 | Failure streaks in the sidebar | P1 | Pending |
| REQ-007 | README and ROADMAP | P2 | Pending |

## Grilling Log

Decisions surfaced and resolved during the 2026-11-02 conversation.

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | Scope | The six remaining sources, as a separate plan after `crawler-core` | Port by slices, user's choice |
| 2 | Single-job fetch | A source without one leaves the job "Sem nota" | Never discard without positive evidence |
