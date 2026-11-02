---
feature: crawler-core
phase: implement
plan_path: docs/plans/2026-11-02-crawler-core.md
started_at: 2026-11-02
spec_path: docs/specs/crawler-core/SPEC.md
---

# STATE: crawler-core

## Current Phase

plan-complete

## Session Log

- 2026-11-02: spec drafted from the conversation decisions; waiting for the user's approval.
- 2026-11-02: spec approved by the user (hard rejects only counted; rescored jobs below the cut stay; opener and crawler.json in scope).
- 2026-11-02: plan saved (10 tasks); validate_plan.py OK. Not implemented.
- 2026-11-02: /rpi-implement --agentic by Claude (Grok, Codex and agy unavailable, exit 3). Task 1 redone: full parity vectors (160 fingerprints, 15 URLs), faithful port of title_key/company_key, url_key now keeps an explicit root path and maps any scheme to https like Python.
- 2026-11-02: Task 6 live check of one Gupy job page: `props.pageProps.job` has `workplaceType` ("remote"), `addressCity`, `addressStateShortName`, `addressCountry` (strings, city and state may be empty), `publishedAt` (ISO with Z) and also `relevantExperiences`, which holds real requirements such as the contract type. Adaptation: `parse_detail` joins `relevantExperiences` too, on top of the plan's three fields, and falls back to `addressCountry` for the location.
- 2026-11-02: Task 7 adaptation: `ScoreUpdate` has no `url_key`. The plan's `APPLY_SCORE` never writes it, and updating it could hit the unique index, so a field that did nothing was dropped.
- 2026-11-02: Task 8 adaptations: the plan's `rescoring_keeps_user_edits_and_stage` passed a `CardDetails` default, and `update_card` replaces every field, so the card lost its `job_url`; the test now keeps the URL. New jobs with a non-http link are rejected as "dados inválidos" (the import applies the same check), and errors that belong to no source are reported under "banco".
- 2026-11-02: Task 9 adaptations: the board listens to a window event named `crawler-changed` (it fires during the run as well, so "finished" would mislead); `CrawlerCard` also lists each failed source with its reason, otherwise the breaker's message would never reach the user; `spawn_worker` emits `crawler-finished` after the ticket drops, also on a panic in dev builds (release uses `panic = "abort"`, so a panic ends the app and the next launch shows the run as interrupted). The Discarded page reloads only when opened, not during a crawl.
