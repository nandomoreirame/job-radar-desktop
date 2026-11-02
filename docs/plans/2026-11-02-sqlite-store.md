# Plan: SQLite Store

> **Spec:** docs/specs/sqlite-store/SPEC.md

**Goal:** Persist cards, moves and edits in a SQLite file behind the existing `Store` trait, so the board survives a restart.
**Architecture:** A `SqliteStore` (one `Mutex<rusqlite::Connection>`) implements `Store` with a normalized schema (`cards`, `timeline_events`) and `PRAGMA user_version` migrations. A shared contract suite runs against `MockStore` and `SqliteStore`. Tauri `setup` resolves the database path per build (`dev.nandomoreira.jobradar-local` in debug, `dev.nandomoreira.jobradar` in release, `JOB_RADAR_DB` overrides), seeds fictional cards only in debug, and a failure shows an `rfd` error dialog and exits 1. IPC commands, bindings and the React app do not change.
**Tech Stack:** Rust 2024, Tauri 2.12, rusqlite 0.41 (`bundled`), rfd 0.17 (`gtk3`), tempfile 3 (dev), serde_json.
**Total Tasks:** 8
**Estimated Complexity:** medium
**Lessons applied:** none

Conventions for every task:
- Rust commands run from `src-tauri/` (`cd src-tauri && cargo test`): running from the root with `--manifest-path` skips `src-tauri/.cargo/config.toml` and writes the ts-rs bindings to the wrong folder.
- Every SQL statement uses bound parameters; no string-built SQL with values.
- Code and comments in English; no `unwrap()`/`expect()` outside tests.
- After each task: `cd src-tauri && cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` stay green.

Batches: B1 = Tasks 1-2 (error type, contract suite), B2 = Tasks 3-4 (schema, SqliteStore), B3 = Tasks 5-7 (path, persistence, startup), B4 = Task 8 (docs).

---

## Checks

**Run rust**: `cd src-tauri && cargo test 2>&1`

- C1 (REQ-001): a move and an edit are listed after the database is reopened. Proof: run rust | sqlite_store::tests::moves_and_edits_survive_a_reopen ... ok
- C2 (REQ-002): SqliteStore passes the same contract as MockStore. Proof: run rust | sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok
- C3 (REQ-002): MockStore still passes the contract. Proof: run rust | store::tests::mock_store_honors_the_store_contract ... ok
- C4 (REQ-003): a fresh database is migrated to the latest version, and reopening does not re-run it. Proof: run rust | sqlite_store::tests::migrates_once_to_the_latest_version ... ok
- C5 (REQ-003): a newer schema is refused and left untouched. Proof: run rust | sqlite_store::tests::refuses_a_newer_schema_without_touching_it ... ok
- C6 (REQ-004): the path follows the build and the env override. Proof: run rust | db_path::tests::picks_the_directory_for_the_build ... ok
- C7 (REQ-004): a non-empty `JOB_RADAR_DB` wins, an empty one is ignored. Proof: run rust | db_path::tests::env_override_wins_unless_empty ... ok
- C8 (REQ-004): the directory is created 0700 and the file 0600, WAL included. Proof: run rust | sqlite_store::tests::database_files_are_private ... ok
- C9 (REQ-005): debug seeds an empty database once; release never seeds. Proof: run rust | tests::open_store_seeds_only_debug_and_only_when_empty ... ok
- C10 (REQ-006): a failed seed leaves no card behind. Proof: run rust | sqlite_store::tests::a_failed_write_leaves_the_data_intact ... ok
- C11 (REQ-006): an unknown stored enum is a storage error, not a panic. Proof: run rust | sqlite_store::tests::unknown_stored_values_are_a_storage_error ... ok
- C12 (REQ-006): the webview sees "storage error" without SQL detail. Proof: run rust | store::tests::storage_errors_hide_the_detail ... ok
- C13 (REQ-007): a database that cannot be opened returns the cause with the path, never a mock store. Proof: run rust | tests::open_store_reports_the_path_and_cause ... ok
- C14 (REQ-002): the store work leaves the TypeScript bindings unchanged. Proof: `(cd src-tauri && cargo test -q) && git diff --exit-code -- src/shared/bindings`
- C15 (REQ-004): a non-Unicode `JOB_RADAR_DB` is kept, not replaced by the default database. Proof: run rust | db_path::tests::env_override_keeps_a_non_unicode_path ... ok
- C16 (REQ-004): startup resolves the override through the same path function. Proof: run rust | tests::open_configured_store_honors_the_env_override ... ok
- C17 (REQ-006): every IPC command logs the storage detail and hides it from the webview. Proof: run rust | commands::tests::every_command_logs_the_storage_detail_and_hides_it ... ok
- C18 (REQ-007): the failure dialog is skipped when there is no display, so the process can exit. Proof: run rust | tests::dialog_needs_a_display ... ok

---

### Task 1: Storage error variant and dependencies

**Requirement:** REQ-006
**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/commands.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

Append inside `mod tests` in `src-tauri/src/store.rs`:

```rust
    #[test]
    fn storage_errors_hide_the_detail() {
        let error = StoreError::Storage("no such table: cards".into());
        assert_eq!(error.to_string(), "storage error");
        assert_eq!(error.detail(), "no such table: cards");
        assert_eq!(StoreError::NotFound("c01".into()).detail(), "card not found: c01");
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test storage_errors_hide_the_detail 2>&1 | tail -5`
Expected: FAIL to compile with "no variant or associated item named `Storage` found for enum `StoreError`"

**Step 3: Write minimal implementation**

In `src-tauri/Cargo.toml`, add under `[dependencies]` and a new `[dev-dependencies]`:

```toml
rusqlite = { version = "0.41", features = ["bundled"] }
rfd = { version = "0.17", default-features = false, features = ["gtk3"] }

[dev-dependencies]
tempfile = "3"
```

In `src-tauri/src/store.rs`, add the variant at the end of `StoreError` and an `impl` below the enum:

```rust
    /// Database failure. Displays without detail so SQL and paths never reach
    /// the webview; `detail()` carries the cause for stderr and the startup dialog.
    #[error("storage error")]
    Storage(String),
}

impl StoreError {
    pub fn detail(&self) -> String {
        match self {
            StoreError::Storage(detail) => detail.clone(),
            other => other.to_string(),
        }
    }
}
```

In `src-tauri/src/commands.rs`, add the helper below `check_id` and use it in the three `*_impl` functions instead of `.map_err(|e| e.to_string())`:

```rust
/// Logs the storage detail to stderr; the webview only gets the display text.
fn report(error: StoreError) -> String {
    if let StoreError::Storage(detail) = &error {
        eprintln!("storage error: {detail}");
    }
    error.to_string()
}
```

```rust
use crate::store::{Store, StoreError};
// list_board_impl:  state.store.list_cards().map_err(report)
// move_card_impl:   state.store.move_card(id, stage, outcome).map_err(report)
// update_card_impl: state.store.update_card(id, details).map_err(report)
```

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "storage_errors_hide_the_detail|test result"`
Expected: PASS, `store::tests::storage_errors_hide_the_detail ... ok` and every `test result: ok.`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Store contract suite, run against MockStore

**Requirement:** REQ-002
**Files:**
- Create: `src-tauri/src/store_contract.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/store.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Replace every test in `mod tests` of `src-tauri/src/store.rs` except `storage_errors_hide_the_detail` with:

```rust
    use crate::mock_data::sample_cards;
    use crate::store_contract;

    #[test]
    fn mock_store_honors_the_store_contract() {
        store_contract::check_all(|| Box::new(MockStore::new(sample_cards())));
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test mock_store_honors 2>&1 | tail -5`
Expected: FAIL to compile with "unresolved import `crate::store_contract`"

**Step 3: Write minimal implementation**

Create `src-tauri/src/store_contract.rs`:

```rust
//! Behavior every `Store` backend shares (REQ-002). Each backend's tests call
//! `check_all` with a factory returning a fresh store seeded with `sample_cards()`.

use crate::domain::{CardDetails, Outcome, Salary, Stage};
use crate::mock_data::sample_cards;
use crate::store::{Store, StoreError};

pub fn check_all(fresh: impl Fn() -> Box<dyn Store>) {
    lists_the_seeded_cards_in_order(&*fresh());
    moving_updates_the_stage(&*fresh());
    moving_to_closed_requires_an_outcome(&*fresh());
    an_outcome_outside_closed_is_rejected(&*fresh());
    moving_out_of_closed_clears_the_outcome(&*fresh());
    unknown_cards_are_not_found(&*fresh());
    updating_applies_only_the_details(&*fresh());
}

fn lists_the_seeded_cards_in_order(store: &dyn Store) {
    assert_eq!(store.list_cards().unwrap(), sample_cards());
}

fn moving_updates_the_stage(store: &dyn Store) {
    let moved = store.move_card("c01", Stage::Triage, None).unwrap();
    assert_eq!(moved.stage, Stage::Triage);
    let listed = store.list_cards().unwrap();
    assert_eq!(listed.iter().find(|c| c.id == "c01").unwrap().stage, Stage::Triage);
}

fn moving_to_closed_requires_an_outcome(store: &dyn Store) {
    assert_eq!(store.move_card("c01", Stage::Closed, None), Err(StoreError::OutcomeRequired));
}

fn an_outcome_outside_closed_is_rejected(store: &dyn Store) {
    assert_eq!(
        store.move_card("c01", Stage::Offer, Some(Outcome::Hired)),
        Err(StoreError::UnexpectedOutcome)
    );
}

fn moving_out_of_closed_clears_the_outcome(store: &dyn Store) {
    let closed = store.move_card("c01", Stage::Closed, Some(Outcome::Discarded)).unwrap();
    assert_eq!(closed.outcome, Some(Outcome::Discarded));
    let reopened = store.move_card("c01", Stage::Triage, None).unwrap();
    assert_eq!(reopened.outcome, None);
}

fn unknown_cards_are_not_found(store: &dyn Store) {
    assert_eq!(store.move_card("nope", Stage::Triage, None), Err(StoreError::NotFound("nope".into())));
    assert_eq!(
        store.update_card("nope", CardDetails::default()),
        Err(StoreError::NotFound("nope".into()))
    );
}

fn updating_applies_only_the_details(store: &dyn Store) {
    let before = store.list_cards().unwrap().into_iter().find(|c| c.id == "c01").unwrap();
    let details = CardDetails {
        salary: Some(Salary { pj: Some(30_000), clt: None }),
        recruiter_name: Some("Ana".into()),
        notes: "Ligar na sexta".into(),
        ..CardDetails::default()
    };
    let updated = store.update_card("c01", details.clone()).unwrap();
    let mut expected = before;
    expected.apply(details);
    assert_eq!(updated, expected);
    let listed = store.list_cards().unwrap();
    assert_eq!(listed.iter().find(|c| c.id == "c01").unwrap(), &expected);
}
```

In `src-tauri/src/lib.rs`, below `pub mod store;`:

```rust
#[cfg(test)]
mod store_contract;
```

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "mock_store_honors|test result"`
Expected: PASS, `store::tests::mock_store_honors_the_store_contract ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: Schema migration and SqliteStore open

**Requirement:** REQ-003
**Files:**
- Create: `src-tauri/migrations/001_init.sql`
- Create: `src-tauri/src/sqlite_store.rs`
- Modify: `src-tauri/src/lib.rs`

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, security-reviewer, business-logic-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/sqlite_store.rs` with only the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn user_version(path: &std::path::Path) -> i64 {
        Connection::open(path).unwrap().pragma_query_value(None, "user_version", |r| r.get(0)).unwrap()
    }

    #[test]
    fn migrates_once_to_the_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
        // A second open must not re-run CREATE TABLE (it would fail).
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
    }

    #[test]
    fn refuses_a_newer_schema_without_touching_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        Connection::open(&path).unwrap().pragma_update(None, "user_version", 99).unwrap();
        let Err(StoreError::Storage(detail)) = SqliteStore::open(&path) else {
            panic!("a newer schema must be refused");
        };
        assert!(detail.contains("newer"), "{detail}");
        assert_eq!(user_version(&path), 99);
        let tables: i64 = Connection::open(&path)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tables, 0);
    }
}
```

In `src-tauri/src/lib.rs`, add `pub mod sqlite_store;` after `pub mod mock_data;`.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find type `SqliteStore` in this scope"

**Step 3: Write minimal implementation**

Create `src-tauri/migrations/001_init.sql` with the schema from the spec's Data Model:

```sql
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

Prepend to `src-tauri/src/sqlite_store.rs`:

```rust
//! SQLite backend (default). One connection behind a mutex: the app is
//! single-user and the `Store` trait is synchronous.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::Connection;

use crate::store::StoreError;

/// Append-only: never edit a shipped migration, add the next file instead.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        StoreError::Storage(error.to_string())
    }
}

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
        conn.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }
}

/// Applies the pending migrations in one transaction; refuses a schema newer
/// than this build so an older app never writes over it (REQ-003).
fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let current = usize::try_from(current).unwrap_or(usize::MAX);
    if current > MIGRATIONS.len() {
        return Err(StoreError::Storage(format!(
            "database schema v{current} is newer than this app supports (v{})",
            MIGRATIONS.len()
        )));
    }
    if current == MIGRATIONS.len() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for sql in &MIGRATIONS[current..] {
        tx.execute_batch(sql)?;
    }
    tx.pragma_update(None, "user_version", MIGRATIONS.len() as i64)?;
    tx.commit()?;
    Ok(())
}
```

Note: `lock` is unused until Task 4; add `#[allow(dead_code)]` on it only if clippy fails this task, and remove it in Task 4.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | grep -E "sqlite_store::tests|test result"`
Expected: PASS, `migrates_once_to_the_latest_version ... ok` and `refuses_a_newer_schema_without_touching_it ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: SqliteStore implements Store, with seeding

**Requirement:** REQ-002, REQ-005, REQ-006
**Files:**
- Modify: `src-tauri/src/sqlite_store.rs`

**Depends on:** Tasks 2 and 3.

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Append inside `mod tests` of `src-tauri/src/sqlite_store.rs`:

```rust
    use crate::mock_data::sample_cards;
    use crate::store::Store;
    use crate::store_contract;

    fn seeded() -> SqliteStore {
        let store = SqliteStore::open_in_memory().unwrap();
        assert!(store.seed_if_empty(&sample_cards()).unwrap());
        store
    }

    #[test]
    fn sqlite_store_honors_the_store_contract() {
        store_contract::check_all(|| Box::new(seeded()));
    }

    #[test]
    fn seeds_only_an_empty_database() {
        let store = seeded();
        store.move_card("c01", Stage::Triage, None).unwrap();
        assert!(!store.seed_if_empty(&sample_cards()).unwrap());
        assert_eq!(store.list_cards().unwrap().len(), sample_cards().len());
        assert_eq!(store.list_cards().unwrap()[0].stage, Stage::Triage);
    }

    #[test]
    fn a_failed_write_leaves_the_data_intact() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut cards = sample_cards();
        cards[1].score = 101; // violates CHECK (score BETWEEN 0 AND 100)
        assert!(matches!(store.seed_if_empty(&cards), Err(StoreError::Storage(_))));
        assert!(store.list_cards().unwrap().is_empty());
    }

    #[test]
    fn unknown_stored_values_are_a_storage_error() {
        let store = seeded();
        {
            let conn = store.lock().unwrap();
            conn.pragma_update(None, "ignore_check_constraints", true).unwrap();
            conn.execute("UPDATE cards SET target = 'astronaut' WHERE id = 'c01'", []).unwrap();
        }
        assert!(matches!(store.list_cards(), Err(StoreError::Storage(_))));
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | tail -5`
Expected: FAIL to compile with "the trait bound `SqliteStore: Store` is not satisfied" and "no method named `seed_if_empty`"

**Step 3: Write minimal implementation**

Replace the `use` block at the top of `src-tauri/src/sqlite_store.rs` with:

```rust
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::domain::{Artifacts, BoardCard, CardDetails, Outcome, Salary, Stage, TimelineEvent};
use crate::store::{Store, StoreError, validate_move};
```

Add below `migrate`:

```rust
macro_rules! card_columns {
    () => {
        "id, title, company, source, target, work_model, location, score, stage, outcome, \
         process_step, salary_pj, salary_clt, has_cv, has_coverage, has_interview_prep, \
         last_activity, next_action, next_action_date, ai_running, unread, recruiter_name, \
         recruiter_email, recruiter_linkedin, job_url, application_url, notes"
    };
}

const LIST_CARDS: &str = concat!("SELECT ", card_columns!(), " FROM cards ORDER BY rowid");
const GET_CARD: &str = concat!("SELECT ", card_columns!(), " FROM cards WHERE id = ?1");
const INSERT_CARD: &str = concat!(
    "INSERT INTO cards (",
    card_columns!(),
    ") VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, \
     ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)"
);
const LIST_EVENTS: &str = "SELECT card_id, at, kind, label FROM timeline_events ORDER BY card_id, at, id";
const CARD_EVENTS: &str = "SELECT card_id, at, kind, label FROM timeline_events WHERE card_id = ?1 ORDER BY at, id";
const INSERT_EVENT: &str = "INSERT INTO timeline_events (card_id, at, kind, label) VALUES (?1, ?2, ?3, ?4)";

/// Domain enums are stored as the same snake_case text the JSON bindings use.
fn to_text<T: Serialize>(value: &T) -> Result<String, StoreError> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(text)) => Ok(text),
        _ => Err(StoreError::Storage("enum did not serialize to text".into())),
    }
}

fn enum_at<T: DeserializeOwned>(row: &Row, idx: usize) -> rusqlite::Result<T> {
    let text: String = row.get(idx)?;
    serde_json::from_value(serde_json::Value::String(text))
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, Box::new(e)))
}

fn card_from_row(row: &Row) -> rusqlite::Result<BoardCard> {
    let pj: Option<u32> = row.get(11)?;
    let clt: Option<u32> = row.get(12)?;
    Ok(BoardCard {
        id: row.get(0)?,
        title: row.get(1)?,
        company: row.get(2)?,
        source: row.get(3)?,
        target: enum_at(row, 4)?,
        work_model: enum_at(row, 5)?,
        location: row.get(6)?,
        score: row.get(7)?,
        stage: enum_at(row, 8)?,
        outcome: match row.get::<_, Option<String>>(9)? {
            Some(_) => Some(enum_at::<Outcome>(row, 9)?),
            None => None,
        },
        process_step: row.get(10)?,
        salary: (pj.is_some() || clt.is_some()).then_some(Salary { pj, clt }),
        artifacts: Artifacts { cv: row.get(13)?, coverage: row.get(14)?, interview_prep: row.get(15)? },
        last_activity: row.get(16)?,
        next_action: row.get(17)?,
        next_action_date: row.get(18)?,
        ai_running: row.get(19)?,
        unread: row.get(20)?,
        recruiter_name: row.get(21)?,
        recruiter_email: row.get(22)?,
        recruiter_linkedin: row.get(23)?,
        job_url: row.get(24)?,
        application_url: row.get(25)?,
        notes: row.get(26)?,
        timeline: Vec::new(),
    })
}

fn event_from_row(row: &Row) -> rusqlite::Result<(String, TimelineEvent)> {
    Ok((row.get(0)?, TimelineEvent { at: row.get(1)?, kind: enum_at(row, 2)?, label: row.get(3)? }))
}

fn insert_card(conn: &Connection, card: &BoardCard) -> Result<(), StoreError> {
    let salary = card.salary.as_ref();
    conn.prepare_cached(INSERT_CARD)?.execute(params![
        card.id,
        card.title,
        card.company,
        card.source,
        to_text(&card.target)?,
        to_text(&card.work_model)?,
        card.location,
        card.score,
        to_text(&card.stage)?,
        card.outcome.map(|o| to_text(&o)).transpose()?,
        card.process_step,
        salary.and_then(|s| s.pj),
        salary.and_then(|s| s.clt),
        card.artifacts.cv,
        card.artifacts.coverage,
        card.artifacts.interview_prep,
        card.last_activity,
        card.next_action,
        card.next_action_date,
        card.ai_running,
        card.unread,
        card.recruiter_name,
        card.recruiter_email,
        card.recruiter_linkedin,
        card.job_url,
        card.application_url,
        card.notes,
    ])?;
    let mut insert_event = conn.prepare_cached(INSERT_EVENT)?;
    for event in &card.timeline {
        insert_event.execute(params![card.id, event.at, to_text(&event.kind)?, event.label])?;
    }
    Ok(())
}

fn load_card(conn: &Connection, id: &str) -> Result<BoardCard, StoreError> {
    let mut card = conn
        .prepare_cached(GET_CARD)?
        .query_row(params![id], card_from_row)
        .optional()?
        .ok_or_else(|| StoreError::NotFound(id.to_owned()))?;
    card.timeline = conn
        .prepare_cached(CARD_EVENTS)?
        .query_map(params![id], event_from_row)?
        .map(|row| row.map(|(_, event)| event))
        .collect::<Result<_, _>>()?;
    Ok(card)
}

impl SqliteStore {
    /// Inserts `cards` in one transaction when the database has none; returns
    /// whether it seeded. Only debug builds call it (REQ-005).
    pub fn seed_if_empty(&self, cards: &[BoardCard]) -> Result<bool, StoreError> {
        let mut conn = self.lock()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(false);
        }
        let tx = conn.transaction()?;
        for card in cards {
            insert_card(&tx, card)?;
        }
        tx.commit()?;
        Ok(true)
    }
}

impl Store for SqliteStore {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError> {
        let conn = self.lock()?;
        let mut cards: Vec<BoardCard> =
            conn.prepare_cached(LIST_CARDS)?.query_map([], card_from_row)?.collect::<Result<_, _>>()?;
        let mut events: HashMap<String, Vec<TimelineEvent>> = HashMap::new();
        for row in conn.prepare_cached(LIST_EVENTS)?.query_map([], event_from_row)? {
            let (card_id, event) = row?;
            events.entry(card_id).or_default().push(event);
        }
        for card in &mut cards {
            card.timeline = events.remove(&card.id).unwrap_or_default();
        }
        Ok(cards)
    }

    fn move_card(&self, id: &str, stage: Stage, outcome: Option<Outcome>) -> Result<BoardCard, StoreError> {
        validate_move(stage, outcome)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "UPDATE cards SET stage = ?2, outcome = ?3 WHERE id = ?1",
            params![id, to_text(&stage)?, outcome.map(|o| to_text(&o)).transpose()?],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }

    fn update_card(&self, id: &str, details: CardDetails) -> Result<BoardCard, StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let salary = details.salary.as_ref();
        let changed = tx.execute(
            "UPDATE cards SET salary_pj = ?2, salary_clt = ?3, process_step = ?4, next_action = ?5, \
             next_action_date = ?6, recruiter_name = ?7, recruiter_email = ?8, recruiter_linkedin = ?9, \
             job_url = ?10, application_url = ?11, notes = ?12 WHERE id = ?1",
            params![
                id,
                salary.and_then(|s| s.pj),
                salary.and_then(|s| s.clt),
                details.process_step,
                details.next_action,
                details.next_action_date,
                details.recruiter_name,
                details.recruiter_email,
                details.recruiter_linkedin,
                details.job_url,
                details.application_url,
                details.notes,
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }
}
```

If `seeds_only_an_empty_database` fails because `c01` is not the first seeded card, compare by id instead of index; do not change `sample_cards()`. Remove any `#[allow(dead_code)]` added in Task 3.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | grep -E "sqlite_store::tests|test result"`
Expected: PASS, the four new tests plus Task 3's report `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: Database path per build and private creation

**Requirement:** REQ-004
**Files:**
- Create: `src-tauri/src/db_path.rs`
- Modify: `src-tauri/src/lib.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/db_path.rs` with the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn picks_the_directory_for_the_build() {
        let data = Path::new("/data/share");
        assert_eq!(
            db_path(None, data, false),
            Path::new("/data/share/dev.nandomoreira.jobradar/job-radar.db")
        );
        assert_eq!(
            db_path(None, data, true),
            Path::new("/data/share/dev.nandomoreira.jobradar-local/job-radar.db")
        );
    }

    #[test]
    fn env_override_wins_unless_empty() {
        let data = Path::new("/data");
        assert_eq!(db_path(Some("/tmp/scratch.db"), data, true), Path::new("/tmp/scratch.db"));
        assert_eq!(db_path(Some("   "), data, false), Path::new("/data/dev.nandomoreira.jobradar/job-radar.db"));
    }

    #[test]
    fn release_dir_matches_the_tauri_identifier() {
        let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], RELEASE_DIR);
    }
}
```

In `src-tauri/src/lib.rs`, add `pub mod db_path;` after `pub mod commands;`.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test db_path 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find function `db_path` in this scope"

**Step 3: Write minimal implementation**

Prepend to `src-tauri/src/db_path.rs`:

```rust
//! Where the database lives (REQ-004). Pure path logic plus private file
//! creation, so it is tested without a Tauri app.

use std::io;
use std::path::{Path, PathBuf};

/// Same as `identifier` in tauri.conf.json (a test keeps them equal).
pub const RELEASE_DIR: &str = "dev.nandomoreira.jobradar";
/// Debug builds never touch the release database.
pub const DEBUG_DIR: &str = "dev.nandomoreira.jobradar-local";
pub const DB_FILE: &str = "job-radar.db";
pub const ENV_VAR: &str = "JOB_RADAR_DB";

pub fn db_path(env_override: Option<&str>, data_dir: &Path, debug: bool) -> PathBuf {
    match env_override.map(str::trim).filter(|value| !value.is_empty()) {
        Some(path) => PathBuf::from(path),
        None => data_dir.join(if debug { DEBUG_DIR } else { RELEASE_DIR }).join(DB_FILE),
    }
}

/// Creates the parent directory (0700) and an empty database file (0600) when
/// missing, so SQLite never creates them with the process umask.
pub fn prepare(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        create_private_dir(parent)?;
    }
    match create_private_file(path) {
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        other => other,
    }
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(unix)]
fn create_private_file(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).map(drop)
}

#[cfg(not(unix))]
fn create_private_file(path: &Path) -> io::Result<()> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(path).map(drop)
}
```

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test db_path 2>&1 | grep -E "db_path::tests|test result"`
Expected: PASS, three `db_path::tests::... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: Persistence across a reopen and private files

**Requirement:** REQ-001, REQ-004
**Files:**
- Modify: `src-tauri/src/sqlite_store.rs`

**Depends on:** Task 4, and Task 5 for `db_path::prepare`.

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Append inside `mod tests` of `src-tauri/src/sqlite_store.rs`:

```rust
    #[test]
    fn moves_and_edits_survive_a_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            store.seed_if_empty(&sample_cards()).unwrap();
            store.move_card("c02", Stage::Closed, Some(Outcome::Rejected)).unwrap();
            let details = CardDetails { notes: "Retornar em dezembro".into(), ..CardDetails::default() };
            store.update_card("c03", details).unwrap();
        }
        let reopened = SqliteStore::open(&path).unwrap();
        let cards = reopened.list_cards().unwrap();
        let c02 = cards.iter().find(|c| c.id == "c02").unwrap();
        assert_eq!((c02.stage, c02.outcome), (Stage::Closed, Some(Outcome::Rejected)));
        assert_eq!(cards.iter().find(|c| c.id == "c03").unwrap().notes, "Retornar em dezembro");
        assert_eq!(cards.len(), sample_cards().len());
    }

    #[cfg(unix)]
    #[test]
    fn database_files_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data").join("job-radar.db");
        crate::db_path::prepare(&path).unwrap();
        let store = SqliteStore::open(&path).unwrap();
        store.seed_if_empty(&sample_cards()).unwrap();
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(&path.with_extension("db-wal")), 0o600);
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | tail -5`
Expected: FAIL to compile with "use of undeclared type `CardDetails`" / "`Outcome`" in the test module

**Step 3: Write minimal implementation**

The production code from Task 4 already persists through the file; this task only brings the domain types into the test module. Change the `use` line at the top of `mod tests` to:

```rust
    use super::*;
    use crate::domain::{CardDetails, Outcome, Stage};
    use rusqlite::Connection;
```

If `database_files_are_private` fails on the WAL file mode, the spec assumption (SQLite copies the main file mode to `-wal`) is false: create the `-wal` and `-shm` files with `0600` in `db_path::prepare` as well, and record the change in `docs/specs/sqlite-store/STATE.md`.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | grep -E "survive_a_reopen|are_private|test result"`
Expected: PASS, `moves_and_edits_survive_a_reopen ... ok` and `database_files_are_private ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: Open the store at startup, error dialog on failure

**Requirement:** REQ-004, REQ-005, REQ-007
**Files:**
- Modify: `src-tauri/src/lib.rs`

**Depends on:** Tasks 4 and 5.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, consequences-reviewer, test-reviewer

**Step 1: Write the failing test**

Append to `src-tauri/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[test]
    fn open_store_seeds_only_debug_and_only_when_empty() {
        let dir = tempfile::tempdir().unwrap();
        let release = open_store(&dir.path().join("release.db"), false).unwrap();
        assert!(release.list_cards().unwrap().is_empty());

        let debug_path = dir.path().join("debug.db");
        let debug = open_store(&debug_path, true).unwrap();
        assert_eq!(debug.list_cards().unwrap().len(), mock_data::sample_cards().len());
        debug.move_card("c01", domain::Stage::Triage, None).unwrap();
        drop(debug);
        let again = open_store(&debug_path, true).unwrap();
        assert_eq!(again.list_cards().unwrap()[0].stage, domain::Stage::Triage);
    }

    #[test]
    fn open_store_reports_the_path_and_cause() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.db");
        std::fs::write(&path, b"this is not a sqlite database at all, just text").unwrap();
        let Err(message) = open_store(&path, true) else { panic!("a broken file must not open") };
        assert!(message.contains("broken.db"), "{message}");
        assert!(message.contains("not a database"), "{message}");
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test open_store 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find function `open_store` in this scope"

**Step 3: Write minimal implementation**

Replace the body of `src-tauri/src/lib.rs` above the test module with:

```rust
pub mod commands;
pub mod db_path;
pub mod details;
pub mod domain;
pub mod mock_data;
pub mod sqlite_store;
pub mod store;
#[cfg(test)]
mod store_contract;

use std::path::Path;

use commands::AppState;
use sqlite_store::SqliteStore;
use tauri::Manager;

/// Opens (and in debug builds seeds) the database; the error names the file
/// and the cause for the startup dialog (REQ-004, REQ-005, REQ-007).
pub fn open_store(path: &Path, debug: bool) -> Result<SqliteStore, String> {
    let fail = |cause: String| format!("Could not open the database at {}:\n{cause}", path.display());
    db_path::prepare(path).map_err(|e| fail(e.to_string()))?;
    let store = SqliteStore::open(path).map_err(|e| fail(e.detail()))?;
    if debug {
        store.seed_if_empty(&mock_data::sample_cards()).map_err(|e| fail(e.detail()))?;
    }
    Ok(store)
}

/// No in-memory fallback: the user must know their data is not being saved.
fn startup_failed(message: &str) -> ! {
    eprintln!("job-radar-desktop: {message}");
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("job-radar-desktop")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let debug = cfg!(debug_assertions);
            let env_override = std::env::var(db_path::ENV_VAR).ok();
            let path = db_path::db_path(env_override.as_deref(), &app.path().data_dir()?, debug);
            let store = open_store(&path, debug)?;
            app.manage(AppState { store: Box::new(store) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_board,
            commands::move_card,
            commands::update_card,
            commands::window_controls
        ])
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|_, _| {}),
        Err(error) => startup_failed(&error.to_string()),
    }
}
```

`open_store(...)?` inside `setup` converts the `String` into `Box<dyn Error>`; if the compiler rejects it, use `.map_err(|m| -> Box<dyn std::error::Error> { m.into() })?`.

Manual check (record the result in `docs/specs/sqlite-store/STATE.md`):
1. `pnpm tauri dev`: the board shows the fictional cards; `ls -l ~/.local/share/dev.nandomoreira.jobradar-local/` lists `job-radar.db` with `-rw-------`.
2. Move a card, close the window, run `pnpm tauri dev` again: the move is still there.
3. `printf 'garbage' > /tmp/jr-broken.db && JOB_RADAR_DB=/tmp/jr-broken.db pnpm tauri dev`: an error dialog names `/tmp/jr-broken.db`; after OK the process exits with status 1 and stderr has the same message. If the dialog does not render under Niri, note it in STATE.md (the spec accepts stderr plus exit 1 as the fallback).

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "tests::open_store|test result"`
Expected: PASS, `tests::open_store_seeds_only_debug_and_only_when_empty ... ok` and `tests::open_store_reports_the_path_and_cause ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 8: README and ROADMAP describe persistence

**Requirement:** REQ-008
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
grep -n "live in memory\|ficam em memória\|There is no$\|Ainda não há banco" README.md README.pt-br.md
```

**Step 2: Run test to verify it fails**
Run: `grep -c "live in memory\|ficam em memória" README.md README.pt-br.md`
Expected: FAIL (the docs are stale): `README.md:2` and `README.pt-br.md:2`

**Step 3: Write minimal implementation**

`README.md`:
- Lines 9-10 become: `> **Status: pre-alpha.** Cards, moves and edits are saved in a local SQLite file; there is no` / `> crawler or AI integration yet; see [Roadmap](#roadmap).`
- Line 47 becomes: `A frameless, maximized window opens. A development build starts with fictional jobs; a release build starts empty.`
- Line 62 becomes: `Everything is saved in SQLite at ~/.local/share/dev.nandomoreira.jobradar/job-radar.db (development builds use dev.nandomoreira.jobradar-local). Set JOB_RADAR_DB to use another file.` (wrap the paths and the variable in backticks).
- Line 102 becomes: `- **Local data only.** Jobs live in a SQLite file in your data directory, created readable only by you (0600); the browser storage keeps only the visual mode.`
- Line 86 becomes: `- An optional Postgres store on a self-hosted server.`

`README.pt-br.md`, same edits in Portuguese:
- Lines 9-10: `> **Status: pré-alpha.** Cards, movimentações e edições ficam salvos num arquivo SQLite local; ainda` / `> não há crawler nem integração com IA; veja o [Roadmap](#roadmap).`
- The "janela" sentence after the quick start: `Abre uma janela sem bordas e maximizada. O build de desenvolvimento começa com vagas fictícias; o de release começa vazio.`
- Line 63: `Tudo fica salvo em SQLite em ~/.local/share/dev.nandomoreira.jobradar/job-radar.db (o build de desenvolvimento usa dev.nandomoreira.jobradar-local). Use JOB_RADAR_DB para apontar outro arquivo.` (paths and variable in backticks).
- Line 104: `- **Dados só locais.** As vagas ficam num arquivo SQLite no seu diretório de dados, criado legível só por você (0600); o armazenamento do navegador guarda só o modo visual.`
- Line 87: `- Store opcional em Postgres num servidor próprio.`

`ROADMAP.md`:
- The bar becomes `██████░░░░░░░░░░░░░░  30%  (3 of 11 milestones, weighted by effort)`.
- Milestone 3 status becomes `Done (independent review pending)` and the total row `**30 done**`.

Then run `python3 ~/.claude/skills/readme-writer/scripts/check_readme.py README.md` and the same for `README.pt-br.md`; fix every FAIL.

**Step 4: Run test to verify it passes**
Run: `grep -c "live in memory\|ficam em memória" README.md README.pt-br.md; grep -n "30%" ROADMAP.md`
Expected: PASS, both counts `0` and the 30% bar line printed

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
