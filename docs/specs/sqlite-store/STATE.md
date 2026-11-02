---
feature: sqlite-store
phase: implement
plan_path: docs/plans/2026-11-02-sqlite-store.md
spec_path: docs/specs/sqlite-store/SPEC.md
started_at: 2026-11-02
---

# STATE: sqlite-store

## Current Phase

plan-complete

## Context

ROADMAP milestone 3: a SQLite `Store` so cards and edits survive a restart, replacing
`MockStore` in the app while keeping the `Store` trait, the IPC commands and the UI unchanged.

## Session Log

- 2026-11-02: research started; open decisions asked to the user (crate, seed, schema, DB path).
- 2026-11-02: grilling done (8 decisions); spec approved by the user.
- 2026-11-02: plan saved (8 tasks, 4 batches); validate_plan.py OK after 2 fixes (task order 5/6, malformed check C14).
- 2026-11-02: Tasks 1-8 implemented and committed: storage errors, Store contract, migrations,
  SqliteStore persistence, private database paths, startup wiring and README/Roadmap updates.
- 2026-11-02: Manual startup validation remains pending: `pnpm tauri dev`, persistence across
  restart, private file permissions and the native error dialog for a corrupt database.
- 2026-11-02: Independent review was attempted through agy, Codex and Grok; all returned exit 3
  for auth/quota/rate-limit/network. `docs/reviews/sqlite-store.md` records zero code findings
  as a self-assessment, not an external review. Keep the phase open until review and manual
  startup validation are available.
- 2026-11-02: The review round identified and fixed eight findings: newer-schema byte mutation,
  missing WAL permission assertion, startup setup panic path, unsafe update error mapping, stale
  formatter coverage, and related proof/documentation gaps. Rust now passes 59 tests, clippy and
  fmt; the independent reviewer chain remains unavailable.
- 2026-11-02: A follow-up startup review found the GTK dialog could block after Tauri build;
  startup now resolves the data directory and opens SQLite before constructing Tauri. Non-Unicode
  JOB_RADAR_DB paths are preserved, the spec/roadmap were corrected, and the proof set is green.
- 2026-11-02: review round 6 by Claude, with fixes authorized by the user: the opencode rounds 1, 3
  and 5 closed with zero findings as self-review. Fixed: C6/C7 tested a function production no
  longer called (one `db_path` over `OsStr` now), hang with no display, `report` test that could
  not fail, duplicate `dirs` major, ROADMAP formatting, spec claims. Measured with the real binary
  and an invalid database: dialog renders under Niri, exit 1 after closing it, exit 1 with no
  display. Manual persistence through the UI is still pending; agy, Codex and Grok unavailable.
- 2026-11-02: round 6 recorded zero findings as a self-assessment (Claude fixed and judged); phase
  stays `implement` until an independent review, the manual UI check and `check_proofs.py` on a
  clean tree.
