# Spec: SQLite Store

## Problem Statement

Every move and edit on the board lives in `MockStore`, an in-memory `Vec` seeded with fictional
cards, so it is gone when the app restarts. Nothing after this milestone (crawler, manual job
entry, AI actions) is usable for a real job search until the board survives a restart.

## Goals

- Cards, moves and edits survive closing and reopening the app.
- `SqliteStore` implements the existing `Store` trait; IPC commands, bindings and the React app
  do not change.
- The database lives in the user's data directory, readable only by the user.
- Development keeps the fictional board; a release build starts empty.

## Context

ROADMAP milestone 3. `board-prototype` set the decision "SQLite default + Postgres optional
behind `Store`; only the trait + mock now". The trait is synchronous and the crate has no async
runtime. Decisions taken with the user on 2026-11-02: `rusqlite` with the `bundled` feature;
fictional seed only in debug builds; normalized schema (cards plus timeline events); debug and
release builds keep separate data directories (`dev.nandomoreira.jobradar-local` and
`dev.nandomoreira.jobradar`), both overridable by `JOB_RADAR_DB`; a startup failure shows a native
error dialog through `rfd`.

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| `Store` is a sync trait with `list_cards`, `move_card`, `update_card`, bound `Send + Sync` | `src-tauri/src/store.rs:21-31` | yes |
| `validate_move` holds the stage/outcome rule outside any backend | `src-tauri/src/store.rs:34-40` | yes |
| `StoreError` derives `PartialEq, Eq` and has `NotFound`, `OutcomeRequired`, `UnexpectedOutcome`, `Poisoned` | `src-tauri/src/store.rs:9-19` | yes |
| The app manages `AppState { store: Box<dyn Store> }` with `MockStore::new(sample_cards())` before `run` | `src-tauri/src/lib.rs:11-14` | yes |
| `BoardCard` has 26 fields, including `salary: Option<Salary>`, `artifacts: Artifacts`, `timeline: Vec<TimelineEvent>`, `notes: String` | `src-tauri/src/domain.rs:103-135` | yes |
| Domain enums serialize as snake_case strings | `src-tauri/src/domain.rs:7-75` | yes |
| The Tauri identifier is `dev.nandomoreira.jobradar`, so the Linux app data dir is `~/.local/share/dev.nandomoreira.jobradar` | `src-tauri/tauri.conf.json:5` | yes |
| `rusqlite` 0.41.1 is the current release and its `bundled` feature compiles SQLite into the binary | `cargo search rusqlite` on 2026-11-02 | yes |
| SQLite gives `-wal`/`-shm` files the permissions of the main database file | SQLite `unix` VFS (`robust_open` copies the db mode) | assumption (checked in a test) |
| `rfd` 0.17.2 is current; its `gtk3` feature (no default features) shows message dialogs through GTK instead of the `zenity` binary used by the default `xdg-portal` backend | `cargo info rfd` on 2026-11-02 | yes (feature list); dialog rendering is an assumption checked manually in the startup task |
| The `setup` hook runs inside `App::run` and a failure there panics; it never surfaces from `Builder::build` | `tauri-2.12` `src/app.rs:1443` | yes (assumption from the grilling refuted) |
| After `Builder::build`, `rfd` (gtk3) blocks forever: its idle callback never runs while tao owns GTK | binary run with an invalid database on 2026-11-02: no window, main thread in `run_blocking` | yes |
| `rfd` also blocks forever when GTK cannot start (no display) | binary run without `DISPLAY`/`WAYLAND_DISPLAY` on 2026-11-02 | yes |
| Opened before `tauri::Builder`, the dialog renders under Niri and the process exits 1 when it closes | binary run with an invalid database on 2026-11-02 | yes (manual; `run()` has no automated test) |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN a card is moved or edited and the app is reopened on the same database file THEN the system SHALL list the card with the new stage, outcome and details.
2. **[REQ-002]** WHEN any `Store` method runs on `SqliteStore` THEN it SHALL behave as `MockStore` does for the same input: the same `validate_move` rule, `NotFound` for an unknown id, a cleared outcome when a card leaves Closed, and cards listed in insertion order with each timeline ordered by date.
3. **[REQ-003]** WHEN the store opens a database THEN the system SHALL apply the pending schema migrations tracked by `PRAGMA user_version` inside one transaction, and SHALL refuse a database whose version is newer than the app knows, with a storage error and no change to the file.
4. **[REQ-004]** WHEN the app starts THEN the system SHALL open the database at `$JOB_RADAR_DB` when that variable is set and non-empty, otherwise at `<data_dir>/<dir>/job-radar.db` where `<dir>` is `dev.nandomoreira.jobradar` in a release build and `dev.nandomoreira.jobradar-local` in a debug build, creating the parent directory with mode `0700`; on Unix a newly created database file SHALL have mode `0600`.
5. **[REQ-005]** WHEN a debug build opens a database with no cards THEN the system SHALL insert the fictional `sample_cards()` in one transaction; a release build SHALL never seed.
6. **[REQ-006]** WHEN a write fails midway or a stored value cannot be read back into the domain (unknown enum text, out-of-range number) THEN the system SHALL leave the previous data intact (one transaction per write) and SHALL return a storage error instead of panicking.
7. **[REQ-007]** WHEN the database cannot be opened or migrated at startup THEN the app SHALL print the cause to stderr, SHALL show a native error dialog naming the cause and the database path when a display is available, SHALL exit with status 1, and SHALL NOT fall back to the in-memory store.

### P2 (Next)

1. **[REQ-008]** WHEN the store ships THEN README (en, pt-BR) and ROADMAP SHALL describe persistence and the database location instead of "changes live in memory", and the ROADMAP bar SHALL move to 30%.

### Out of Scope

- Postgres backend and any change to the `Store` trait signature.
- Creating or deleting cards (milestone 5 adds manual entry; the crawler is milestone 4).
- Timeline events written by `move_card` (the mock does not write them either).
- Backup, export, import or encryption of the database file.
- A settings UI for the database path.

## Edge Cases

- **`JOB_RADAR_DB` set but empty** - ignored; the default path is used.
- **Database file is not SQLite or is corrupt** - open fails, startup dialog and exit 1 (REQ-007); the file is not touched.
- **No display (`DISPLAY` and `WAYLAND_DISPLAY` unset or empty)** - startup failure skips the dialog, prints to stderr and exits 1. A variable set to a dead display still blocks inside `rfd`; accepted.
- **Non-Unicode `JOB_RADAR_DB`** - used byte for byte; only an empty or blank value falls back to the default path.
- **Database schema newer than the app** - refused before any write (REQ-003), same startup path.
- **Debug build, database already has cards** - no seed; dev keeps what the developer did.
- **Stored enum text the app does not know** (hand-edited file) - `list_cards` returns a storage error; nothing panics.
- **Card with no timeline events** - listed with an empty `timeline`.
- **Both salary columns NULL** - `salary` is `None`; one set means `Some` with the other `None`.
- **Move to closed without an outcome** - `validate_move` rejects it before any SQL runs; the CHECK constraint is a second guard.

## Data Model

```sql
-- migration 1
CREATE TABLE cards (
  id                 TEXT PRIMARY KEY CHECK (length(id) BETWEEN 1 AND 64),
  title              TEXT NOT NULL,
  company            TEXT NOT NULL,
  source             TEXT NOT NULL,
  target             TEXT NOT NULL CHECK (target IN ('ai_llm','full_stack','tech_lead','data')),
  work_model         TEXT NOT NULL CHECK (work_model IN ('remote','hybrid','onsite')),
  location           TEXT,
  score              INTEGER NOT NULL CHECK (score BETWEEN 0 AND 100),
  stage              TEXT NOT NULL CHECK (stage IN ('inbox','triage','applied','in_process','offer','closed')),
  outcome            TEXT CHECK (outcome IN ('hired','rejected','discarded','no_response')),
  process_step       TEXT,
  salary_pj          INTEGER CHECK (salary_pj >= 0),
  salary_clt         INTEGER CHECK (salary_clt >= 0),
  has_cv             INTEGER NOT NULL DEFAULT 0 CHECK (has_cv IN (0,1)),
  has_coverage       INTEGER NOT NULL DEFAULT 0 CHECK (has_coverage IN (0,1)),
  has_interview_prep INTEGER NOT NULL DEFAULT 0 CHECK (has_interview_prep IN (0,1)),
  last_activity      TEXT,
  next_action        TEXT,
  next_action_date   TEXT,
  ai_running         TEXT,
  unread             INTEGER NOT NULL DEFAULT 0 CHECK (unread IN (0,1)),
  recruiter_name     TEXT,
  recruiter_email    TEXT,
  recruiter_linkedin TEXT,
  job_url            TEXT,
  application_url    TEXT,
  notes              TEXT NOT NULL DEFAULT '',
  CHECK ((stage = 'closed') = (outcome IS NOT NULL))
);

CREATE TABLE timeline_events (
  id      INTEGER PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  at      TEXT NOT NULL,
  kind    TEXT NOT NULL CHECK (kind IN ('found','applied','response','interview','offer','closed')),
  label   TEXT NOT NULL
);
CREATE INDEX timeline_events_card ON timeline_events(card_id, at);
```

- `salary` maps to `Some(Salary)` when either column is set, `None` when both are NULL.
- Insertion order is `rowid` order; timeline order is `at, id`.
- Connection pragmas on open: `foreign_keys = ON`, `journal_mode = WAL`, `busy_timeout = 5000`.

## Architecture Decisions

1. **`rusqlite` with `bundled`, one `Mutex<Connection>`** - The trait is sync and the app is
   single-user; a mutex over one connection matches `MockStore` and needs no pool or runtime.
   `bundled` avoids depending on the system `libsqlite3` version in AppImage/deb builds.
2. **`PRAGMA user_version` migrations in code** - One `MIGRATIONS: &[&str]` array, applied in a
   transaction; no extra crate. A newer version than `MIGRATIONS.len()` is refused so an older
   app never writes over a newer schema.
3. **Enums as snake_case TEXT with CHECK constraints** - Same strings as the JSON bindings, read
   back through serde so there is one mapping. Unknown text becomes a storage error.
4. **One contract test suite run against both stores** - A generic test helper takes any
   `Store`, so `MockStore` and `SqliteStore` cannot drift (REQ-002).
5. **Storage errors hide SQL detail from the UI** - `StoreError::Storage(String)` displays as
   "storage error"; the detail goes to stderr. The webview never sees SQL or file paths.
6. **Store built before the Tauri event loop** - `dirs::data_dir()` resolves the user's data
   directory before GTK/Tauri startup, so a corrupt database can show the native error dialog and
   exit instead of blocking inside a `setup` hook. The command layer keeps reading
   `State<AppState>`.
7. **Separate dev data directory in code, not in config** - `cfg!(debug_assertions)` picks the
   `-local` directory, so `pnpm tauri dev` can never write fictional cards into the release
   database and no `--config` flag has to be remembered.
8. **Startup failure dialog through `rfd` (gtk3), before Tauri build** - No Tauri plugin and no
   new webview permission; the dialog runs before the event loop can claim the GTK main context.

## Security

- The database holds salary expectations, recruiter contacts and notes: file mode `0600` and
  parent directory created with `0700` on Unix.
- All SQL uses bound parameters; no string-built queries.
- `JOB_RADAR_DB` is read once at startup from the user's own environment; it is not reachable
  from the webview.
- The CSP and Tauri permissions do not change: the webview gets no file system or SQL access.

## Success Criteria

- Moving and editing a card, closing the app and reopening it shows the change.
- The shared contract suite passes for both stores.
- `pnpm tauri dev` shows the fictional board; a release build on a fresh data dir shows an empty
  board.
- `cd src-tauri && cargo test`, clippy, and the web gates stay green; bindings are unchanged.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| `bundled` raises build time and binary size | Low | Accepted; release profile already uses LTO and strip |
| A future schema change breaks existing user data | High | Append-only migrations, refusal of newer versions, a test that opens a v1 file |
| `rfd` dialog fails to render on a given desktop | Low | stderr always carries the cause first; with no display the dialog is skipped and the exit status is 1 |

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | Moves and edits survive a reopen | P1 | Pending |
| REQ-002 | SqliteStore matches the MockStore contract | P1 | Pending |
| REQ-003 | `user_version` migrations, newer schema refused | P1 | Pending |
| REQ-004 | Database path per build, env override, 0700/0600 | P1 | Pending |
| REQ-005 | Fictional seed only in debug, only when empty | P1 | Pending |
| REQ-006 | Atomic writes, storage error instead of panic | P1 | Pending |
| REQ-007 | Startup failure dialog, exit 1, no mock fallback | P1 | Pending |
| REQ-008 | README and ROADMAP describe persistence | P2 | Pending |

## Grilling Log

Decisions surfaced and resolved during the grilling pass (pre-build stress test).

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | SQLite crate | `rusqlite` 0.41 `bundled`, one `Mutex<Connection>` | Sync trait, no runtime, no system libsqlite dependency |
| 2 | First-run content | Seed `sample_cards()` only in debug builds and only into an empty database | Dev keeps a populated board; release starts clean |
| 3 | Schema shape | Normalized `cards` + `timeline_events`, enums as TEXT with CHECK, `user_version` migrations | Metrics and the crawler need queryable rows |
| 4 | Database location | Debug `dev.nandomoreira.jobradar-local`, release `dev.nandomoreira.jobradar` under the data dir, file `job-radar.db`; `JOB_RADAR_DB` overrides both | Dev can never pollute the real database (user's choice) |
| 5 | Startup failure UX | Native `rfd` (gtk3) dialog with cause and path, stderr detail, exit 1, no mock fallback | A launcher start must not fail silently; no frontend or IPC change |
| 6 | Timeline on `move_card` | Not written, same as `MockStore` | Parity; timeline writes belong to a later feature |
| 7 | Error text to the webview | `storage error` only; detail to stderr | No SQL or paths in the UI |
| 8 | File permissions | Directory `0700`, database `0600` | Salary, recruiter contacts and notes are personal data |
