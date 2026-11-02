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
- 2026-11-02: Task 10 manual check, run headless through a temporary example over scratch copies of the server's data (all deleted afterwards), with a real profile and the live sites:
  - Run over the imported database (207 inbox, 138 discarded): found 198 (gupy 147, solides 51), no source error, 70 known, 102 rejected by reason, 307 rescored, 4 closed, 0 failed, 34 skipped (unported sources); took 5 min. No new job because the server had crawled at 18:08; on an empty database the same run put 38 in the Inbox (scores 70-100, unread) and 32 in Discarded (scores 5-65), in 2 min 20 s.
  - Scores: 138 of 138 stored-description jobs equal the Python `score_job` on the same text and profile.
  - Closed sample: only 1 Gupy job closed, and it answers 404 live, so the plan's sample of 5 was not reachable. Of 3 Solides jobs, 2 answer 404. The third, `vaga/553615`, answered 200 with "JobPosting" in one request and 404 in eight others (three user agents), so Solides flaps on that page; cause not established, no code changed. A wrong close can be undone with "move to Inbox" on Discarded.
  - Interrupted: killing the process leaves the run with no `finished_at`; the next run completes and its row is the latest.
  - The Solides 403 path was not seen live (0 failures); it is covered by `detail_outcomes_follow_the_closed_rules`.
  - NOT done: the desktop window itself. "Executar agora", live progress, the refused message and "interrompida" on screen were only proved by component tests; `start_crawl` and the events never ran inside a real Tauri window.
