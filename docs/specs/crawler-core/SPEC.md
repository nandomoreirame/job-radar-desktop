# Spec: Crawler Core

## Problem Statement

"Executar agora" only says "not implemented yet", and imported jobs have no score. Today finding jobs
needs a Python crawler running on a server, which an open source user of this app does not have.
Without a crawler in the app, the board goes stale the day after the import.

## Goals

- The app finds new jobs by itself, from "Executar agora", with no server and no Python.
- Rust gives the same identity, filter and score results as the Python crawler for the same input.
- Imported jobs get a real score from their job page; closed ads leave the Inbox.
- The sidebar shows the real crawler status instead of the fixed sample text.

## Context

Depends on `real-jobs-import` (nullable fields, `discard_reason`, `dismissed_jobs`). The reference
is the author's public MIT repo `nandomoreirame/job-radar` (`../job-radar` next to this repo). This
slice covers the shared core plus Gupy and Solides, which hold 173 of the 207 imported jobs;
`crawler-sources` covers the other six. Decided with the user on 2026-11-02: port to Rust in slices;
recalculate scores from the job page; HTTP 404 or 410 discards the job as a closed ad; timeouts,
blocks and unported sources leave it as "no score" for a later retry.

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| `job_fingerprint` is `sha256("company_key|title_key")[:16]`, title only when the company is empty | `job_radar/identity.py:245-250` | yes |
| The core to port is identity (363 lines), filters (314), scoring (69), profile (94), settings (61), http (178), robots (281), dedup (106) | `wc -l` on 2026-11-02 | yes |
| Several filter regexes use lookaround, which the `regex` crate lacks; `fancy-regex` 0.20 has it | `job_radar/filters.py`, `cargo search` | yes |
| A Gupy job page embeds `__NEXT_DATA__` with `props.pageProps.job`, holding `description`, `responsibilities`, `prerequisites`, `publishedAt`, `expiresAt`, address fields | fetch of 3 imported URLs from the server on 2026-11-02 | yes |
| A dead Gupy job answers HTTP 200 with no `pageProps.job`, so 404/410 alone cannot detect it | fetch of an invalid id on 2026-11-02 | yes (one sample) |
| A live Solides page has a JSON-LD `JobPosting` with `description`; a removed one answers 404 | fetch of 3 imported URLs on 2026-11-02 | yes |
| Solides `validThrough` can be past on a page that still serves the job, so it does not mean closed | same fetch: `validThrough` 2026-08-31 on a job sent 2026-09-23 | yes |
| Solides answers bursts with 403, retried by the crawler at 5 s and 10 s | `job_radar/sources/solides.py:43-60` | yes |
| Recorded fixtures for both sources and a sample profile exist | `tests/fixtures/{gupy_search.html,solides_listing.json,solides_jobposting.html,perfil.md}` | yes |
| The webview CSP only concerns the webview; requests from Rust are outside it | `src-tauri/tauri.conf.json` | yes |
| `ureq` 3.5 is a blocking client, matching the synchronous `Store` | `cargo search` on 2026-11-02 | yes (crate; TLS setup checked when the task compiles) |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN Rust computes `fingerprint` and `url_key` THEN the values SHALL equal the Python crawler's for every vector in a committed fixture generated from the Python code.
2. **[REQ-002]** WHEN Rust applies the hard filters and the score to a job THEN the verdict, the reason and the score SHALL equal the Python crawler's for every vector in a committed fixture, with a fixed clock.
3. **[REQ-003]** WHEN a crawl starts THEN the system SHALL read `crawler.json` from the app data directory (profile path, salary floors, commute cities, maximum age, blocked companies, minimum score default 70, minimum skills default 3, neutral defaults otherwise); a missing or skill-less profile SHALL refuse the run with a visible message and no request made; an invalid config SHALL refuse with the field named.
4. **[REQ-004]** WHEN the crawler makes a request THEN it SHALL honor robots.txt, identify itself in the User-Agent, wait at least 1 s or the `Crawl-delay` per host, retry 429 and 5xx with backoff, bound time and body size, follow only http(s) redirects, and never send the profile or any user data.
5. **[REQ-005]** WHEN the Gupy and Solides connectors parse a recorded page THEN they SHALL produce the same jobs as the Python connectors for the same fixture, and a changed page shape SHALL count as a source error, not as zero jobs found.
6. **[REQ-006]** WHEN the user runs a crawl THEN a job new to the database and at or above the minimum score SHALL enter the Inbox unread; one below it SHALL go to the Discarded page with the reason "abaixo do corte de score"; one rejected by a hard filter SHALL only count in the run's statistics by reason; a job matching an existing card or a tombstone by fingerprint or URL key SHALL be skipped; one failing source SHALL NOT stop the others.
7. **[REQ-007]** WHEN a crawl runs and a card has no score THEN the system SHALL score it: from its stored description when it has one, else by fetching its page when the source is supported, filling score, target, work model, location and description, and SHALL leave stage and every user-edited field intact; a rescored job below the cut SHALL stay where it is.
8. **[REQ-008]** WHEN a job page answers 404 or 410, or a Gupy page answers 200 without a job THEN the card SHALL go to the Discarded page with the reason "anúncio encerrado", unless the user already moved it out of the Inbox, in which case only its score stays empty; a timeout, block, parse failure or unsupported source SHALL leave it without a score for a later run; if more than half of at least 10 pages of a source look closed in one run THEN no card of that source SHALL be discarded and the run SHALL report a source error.
9. **[REQ-009]** WHEN a crawl is running THEN "Executar agora" SHALL be disabled, a second start SHALL be refused, the board SHALL refresh as cards change, and the sidebar SHALL show progress; WHEN it ends THEN the sidebar SHALL show the real last run time, duration, totals and sources ok, persisted across restarts, and the fixed sample status and "Próxima" SHALL be gone.
10. **[REQ-010]** WHEN the app has never crawled THEN the sidebar SHALL say so, and a run refused for configuration SHALL say what to fix.

### P2 (Next)

1. **[REQ-011]** WHEN this ships THEN README (en, pt-BR) SHALL replace "No network access" with every contacted host and the data sent, document `crawler.json`, and ROADMAP SHALL show the milestone as partial.

### Out of Scope

- The six other sources and near-duplicate detection (`crawler-sources`).
- Scheduled runs; the crawler runs only on demand.
- A settings UI for `crawler.json` (milestone 6) and a profile page (milestone 7).
- Discord or any other notification.
- Reposted jobs re-entering after a window: a job is known forever once stored.

## Edge Cases

- **Profile without the "Stack técnica" section** - zero skills; the run is refused, as in Python.
- **A job with no company** - fingerprint by title only, as in Python.
- **Robots disallows a page** - counts as blocked, never as closed.
- **Solides 403 after retries** - blocked; the card keeps no score and the source shows an error.
- **Gupy page with a job but an empty description** - parse failure; no score, no discard.
- **Stored description on a job without a fetchable page** - scored without a request.
- **App closed during a crawl** - each job commits on its own; the next run continues; a run with no end time shows as interrupted.
- **Card discarded then restored by the user** - stays restored; a later run does not discard it again unless its page is closed and it is still in the Inbox.

## Data Model

```sql
-- migration 3
CREATE TABLE crawl_runs (
  id          INTEGER PRIMARY KEY,
  started_at  TEXT NOT NULL,
  finished_at TEXT,
  stats       TEXT NOT NULL DEFAULT '{}' -- JSON: totals, per-source status, reasons
);
```

## Architecture Decisions

1. **`crawler/` module tree, blocking I/O on a worker thread** - `ureq` and the sync `Store` need no runtime; the command spawns a thread and emits progress events.
2. **Parity by generated vectors** - A small script in the Python repo's style generates JSON vectors committed as fixtures here; Rust tests read them. Fixtures use fictional data only.
3. **`fancy-regex` only where lookaround is needed** - The rest uses `regex`; both keep Unicode case folding close to Python's.
4. **`crawler.json` before a settings UI** - A launcher gives no environment variables, and milestone 6 can write the same file.
5. **Closed needs positive evidence** - Only 404, 410 or Gupy's jobless page count, with a per-source circuit breaker, because a false positive removes a live job from the Inbox.
6. **Hard rejects are not stored** - About 900 repeat on each run; only counts per reason are kept.

## Security

- First network surface of the app: only source hosts, http(s), bounded responses, no credentials, no user data.
- Page content is untrusted: bound parameters, text bounded, descriptions rendered as plain text, URLs validated before storage.
- The profile is read locally and never sent. `crawler.json` and the profile stay outside the repo.
- The webview gets no network or file access; CSP unchanged.

## Success Criteria

- A crawl from the button adds real Gupy and Solides jobs to the Inbox with scores equal to the Python crawler's for the same text.
- After it, every imported Gupy and Solides job has a score or is on the Discarded page as closed, except those that failed transiently.
- All parity vectors pass; Rust and web gates stay green.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Regex semantics differ from Python's | High | Vectors cover each filter branch, including accents and `PL/SQL`, `java` vs `javascript` |
| Sources change their markup | Medium | Connectors report a source error on a missing payload; fixtures pin today's shape |
| Solides blocks a desktop IP | Medium | Same pacing as Python; failures leave cards untouched |
| Gupy's closed signal rests on one sample | Medium | Circuit breaker; the manual check samples more |
| Rescoring 173 pages takes minutes | Low | Progress in the sidebar; it runs on a worker thread and resumes on the next run |

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | Identity parity | P1 | Pending |
| REQ-002 | Filter and score parity | P1 | Pending |
| REQ-003 | `crawler.json` and profile | P1 | Pending |
| REQ-004 | Polite, bounded HTTP | P1 | Pending |
| REQ-005 | Gupy and Solides connectors | P1 | Pending |
| REQ-006 | Crawl pipeline and routing | P1 | Pending |
| REQ-007 | Rescoring | P1 | Pending |
| REQ-008 | Closed ads and circuit breaker | P1 | Pending |
| REQ-009 | Run control and real sidebar status | P1 | Pending |
| REQ-010 | First-run and refused-run messages | P1 | Pending |
| REQ-011 | README and ROADMAP | P2 | Pending |

## Grilling Log

Decisions surfaced and resolved during the 2026-11-02 conversation.

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | Engine | Rust port by slices | The app must stay self-sufficient for open source users |
| 2 | First slice | Core plus Gupy and Solides | They hold 173 of the 207 imported jobs that need a score |
| 3 | Imported scores | Recalculated from the job page | User's choice |
| 4 | Closed ads | 404, 410 or Gupy's jobless page go to Discarded; other failures stay "Sem nota" | User's choice; extended to Gupy because it answers 200 |
| 5 | Config | `crawler.json` plus a local profile file | The settings UI comes later; nothing personal in the repo |
