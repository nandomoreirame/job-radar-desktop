# Plan: Real Jobs Import

> **Spec:** docs/specs/real-jobs-import/SPEC.md

**Goal:** Replace the fictional dev board with real jobs loaded from the crawler's `seen.db` and `candidates.json`, with a Discarded page for curation and job links that open in the browser.
**Architecture:** Migration 2 makes score, target and work model nullable and adds `url_key`, `description`, `discard_reason`, `discarded_at` to `cards`, plus a `dismissed_jobs` tombstone table. An `import` subcommand of the same binary reads both files read-only and inserts them in one transaction. The webview gets new IPC commands for the Discarded page and opens links by card id only.
**Tech Stack:** Rust 2024, Tauri 2.12, rusqlite 0.41, regex 1, url 2, tauri-plugin-opener 2, React 19, shadcn base-nova, Vitest 5.
**Total Tasks:** 11
**Estimated Complexity:** large
**Lessons applied:** none

Conventions for every task:
- Rust commands run from `src-tauri/` (`cd src-tauri && cargo test`); from the root, ts-rs writes bindings to the wrong folder.
- Web commands run from the repo root with `pnpm`; use `./node_modules/.bin/<tool>`, never a bare tool name.
- After each Rust task: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` stay green. `rpi-gates` only detects TypeScript in this repo, so run them by hand.
- No `unwrap()`/`expect()` outside tests; all SQL uses bound parameters; UI strings only through i18n keys in both locales.
- Fixtures are fictional. Never commit real job data, a server address or an absolute home path.
- Verify runtime behavior by running the binary, not only through helpers (see the sqlite-store review).

Batches: B1 = Tasks 1-3 (nullable fields, no mocks), B2 = Tasks 4-7 (discard store, import), B3 = Tasks 8-10 (link, Discarded page, empty state), B4 = Task 11 (docs and manual check).

---

## Checks

**Run rust**: `cd src-tauri && cargo test 2>&1`
**Run web**: `pnpm vitest run --reporter=verbose`

- C1 (REQ-001): opening an empty database in a debug or release build inserts no card. Proof: run rust | tests::open_store_never_seeds ... ok
- C2 (REQ-001): a non-test build compiles without the fictional cards. Proof: `cd src-tauri && cargo build 2>&1 && ! grep -q 'pub mod mock_data' src/lib.rs`
- C3 (REQ-002): a version 1 database keeps every field after the migration. Proof: run rust | sqlite_store::tests::migration_2_keeps_existing_rows ... ok
- C4 (REQ-002): a card with no score, target or work model round-trips through the store. Proof: run rust | sqlite_store::tests::a_card_without_score_round_trips ... ok
- C5 (REQ-002): a job card without a score shows "Sem nota". Proof: run web | ✓ JobCard shows no score label when the score is null
- C6 (REQ-003): a job without a score passes any minimum score. Proof: run web | ✓ filterCards keeps a card without a score at any minimum
- C7 (REQ-003): the Inbox lists by score, no score last, then newest. Proof: run web | ✓ groupByStage sorts the inbox by score then newest
- C8 (REQ-004): import routes seen rows to the Inbox and candidates to Discarded. Proof: run rust | import::tests::routes_seen_to_inbox_and_candidates_to_discarded ... ok
- C9 (REQ-004): the command line is parsed and a bad flag is refused. Proof: run rust | cli::tests::parses_the_import_command ... ok
- C10 (REQ-005): a second import adds nothing and keeps user edits. Proof: run rust | import::tests::importing_twice_changes_nothing ... ok
- C11 (REQ-005): the same job in both files stays in the Inbox, also by URL key. Proof: run rust | import::tests::seen_wins_over_candidates ... ok
- C12 (REQ-006): invalid records are skipped and counted. Proof: run rust | import::tests::skips_and_counts_invalid_records ... ok
- C13 (REQ-006): a failure leaves the database unchanged. Proof: run rust | import::tests::unreadable_candidates_change_nothing ... ok
- C14 (REQ-006): URL keys equal the crawler's on the recorded cases. Proof: run rust | identity::tests::url_key_matches_the_crawler ... ok
- C15 (REQ-007): a record maps to id, target, work model, event and description. Proof: run rust | import::tests::maps_a_record_to_a_card ... ok
- C16 (REQ-008): the Discarded page lists, searches and shows the description. Proof: run web | ✓ DiscardedPage lists and searches discarded jobs
- C17 (REQ-008): the page passes axe. Proof: run web | ✓ DiscardedPage has no axe violations
- C18 (REQ-009): restore moves the job back to the Inbox. Proof: run web | ✓ DiscardedPage restores a job
- C19 (REQ-009): delete asks for confirmation first. Proof: run web | ✓ DiscardedPage deletes a job only after confirming
- C20 (REQ-009): a deleted job never returns through import. Proof: run rust | import::tests::a_deleted_job_does_not_return ... ok
- C21 (REQ-010): opening uses the stored URL by id and refuses a non-http value. Proof: run rust | commands::tests::opens_only_a_stored_http_url ... ok
- C22 (REQ-010): the dialog opens the link through the API. Proof: run web | ✓ JobDialog opens the job link by id
- C23 (REQ-011): an empty board explains how to bring jobs in. Proof: run web | ✓ BoardPage shows the empty state with no cards
- C24 (REQ-002): the discard store behaves the same on both backends. Proof: run rust | sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok

---

### Task 1: Optional score, target and work model, and migration 2

**Requirement:** REQ-002
**Files:**
- Create: `src-tauri/migrations/002_real_jobs.sql`
- Modify: `src-tauri/src/domain.rs`
- Modify: `src-tauri/src/sqlite_store.rs`
- Modify: `src-tauri/src/mock_data.rs`
- Modify: `src-tauri/src/store_contract.rs`

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, security-reviewer, business-logic-reviewer

**Step 1: Write the failing test**

Append inside `mod tests` of `src-tauri/src/sqlite_store.rs`:

```rust
    #[test]
    fn migration_2_keeps_existing_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATIONS[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            insert_card(&conn, &crate::mock_data::sample_cards()[6]).unwrap();
        }
        let expected = crate::mock_data::sample_cards()[6].clone();
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.list_cards().unwrap(), vec![expected]);
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
        let keys: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM cards WHERE description = '' AND discard_reason IS NULL", [], |r| r.get(0))
            .unwrap();
        assert_eq!(keys, 1);
    }

    #[test]
    fn a_card_without_score_round_trips() {
        let store = SqliteStore::open_in_memory().unwrap();
        let mut card = crate::mock_data::sample_cards().remove(0);
        card.score = None;
        card.target = None;
        card.work_model = None;
        store.seed_if_empty(std::slice::from_ref(&card)).unwrap();
        assert_eq!(store.list_cards().unwrap(), vec![card]);
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test sqlite_store 2>&1 | tail -5`
Expected: FAIL to compile with "expected `u8`, found `Option<_>`"

**Step 3: Write minimal implementation**

`src-tauri/migrations/002_real_jobs.sql`:

```sql
-- Rebuild: SQLite cannot drop NOT NULL. Runs with foreign_keys off (see SqliteStore::init).
CREATE TABLE cards_new (
  id TEXT PRIMARY KEY CHECK (length(id) BETWEEN 1 AND 64),
  title TEXT NOT NULL,
  company TEXT NOT NULL,
  source TEXT NOT NULL,
  target TEXT CHECK (target IN ('ai_llm','full_stack','tech_lead','data')),
  work_model TEXT CHECK (work_model IN ('remote','hybrid','onsite')),
  location TEXT,
  score INTEGER CHECK (score BETWEEN 0 AND 100),
  stage TEXT NOT NULL CHECK (stage IN ('inbox','triage','applied','in_process','offer','closed')),
  outcome TEXT CHECK (outcome IN ('hired','rejected','discarded','no_response')),
  process_step TEXT,
  salary_pj INTEGER CHECK (salary_pj >= 0),
  salary_clt INTEGER CHECK (salary_clt >= 0),
  has_cv INTEGER NOT NULL DEFAULT 0 CHECK (has_cv IN (0,1)),
  has_coverage INTEGER NOT NULL DEFAULT 0 CHECK (has_coverage IN (0,1)),
  has_interview_prep INTEGER NOT NULL DEFAULT 0 CHECK (has_interview_prep IN (0,1)),
  last_activity TEXT,
  next_action TEXT,
  next_action_date TEXT,
  ai_running TEXT,
  unread INTEGER NOT NULL DEFAULT 0 CHECK (unread IN (0,1)),
  recruiter_name TEXT,
  recruiter_email TEXT,
  recruiter_linkedin TEXT,
  job_url TEXT,
  application_url TEXT,
  notes TEXT NOT NULL DEFAULT '',
  url_key TEXT,
  description TEXT NOT NULL DEFAULT '',
  discard_reason TEXT,
  discarded_at TEXT,
  CHECK ((stage = 'closed') = (outcome IS NOT NULL))
);
INSERT INTO cards_new (rowid, id, title, company, source, target, work_model, location, score, stage,
  outcome, process_step, salary_pj, salary_clt, has_cv, has_coverage, has_interview_prep,
  last_activity, next_action, next_action_date, ai_running, unread, recruiter_name, recruiter_email,
  recruiter_linkedin, job_url, application_url, notes)
SELECT rowid, id, title, company, source, target, work_model, location, score, stage,
  outcome, process_step, salary_pj, salary_clt, has_cv, has_coverage, has_interview_prep,
  last_activity, next_action, next_action_date, ai_running, unread, recruiter_name, recruiter_email,
  recruiter_linkedin, job_url, application_url, notes
FROM cards;
DROP TABLE cards;
ALTER TABLE cards_new RENAME TO cards;
CREATE UNIQUE INDEX cards_url_key ON cards(url_key) WHERE url_key IS NOT NULL;
CREATE INDEX cards_discard_reason ON cards(discard_reason);

CREATE TABLE dismissed_jobs (
  fingerprint TEXT PRIMARY KEY,
  url_key TEXT,
  dismissed_at TEXT NOT NULL
);
CREATE INDEX dismissed_jobs_url_key ON dismissed_jobs(url_key);
```

`src-tauri/src/sqlite_store.rs`:

```rust
pub const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_init.sql"),
    include_str!("../migrations/002_real_jobs.sql"),
];
```

Add below `enum_at` and use it for columns 4 and 5 of `card_from_row`, and in the `outcome` match:

```rust
fn opt_enum_at<T: DeserializeOwned>(row: &Row<'_>, idx: usize) -> rusqlite::Result<Option<T>> {
    match row.get::<_, Option<String>>(idx)? {
        Some(_) => enum_at(row, idx).map(Some),
        None => Ok(None),
    }
}
```

```rust
        target: opt_enum_at(row, 4)?,
        work_model: opt_enum_at(row, 5)?,
        score: row.get(7)?,
        outcome: opt_enum_at(row, 9)?,
```

In `insert_card`:

```rust
        card.target.map(|v| to_text(&v)).transpose()?,
        card.work_model.map(|v| to_text(&v)).transpose()?,
```

Keep `LIST_CARDS` for now; Task 4 adds the `discard_reason IS NULL` condition. `seed_if_empty` stays for tests until Task 3.

`src-tauri/src/domain.rs`, in `BoardCard`:

```rust
    /// Null until the crawler scores the job's page.
    pub target: Option<Target>,
    pub work_model: Option<WorkModel>,
    pub location: Option<String>,
    pub score: Option<u8>,
```

`src-tauri/src/mock_data.rs`: in `fn card(...)` set `target: Some(target)`, `work_model: Some(work_model)`, `score: Some(score)`. Keep its parameters.

Do not edit the frontend in this task; `pnpm typecheck` goes red until Task 2, which runs right after it. `store_contract.rs` needs no logic change; only fix any type error the new `Option` fields raise.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "migration_2|without_score|test result"`
Expected: PASS, both tests `... ok`, and `src/shared/bindings/BoardCard.ts` now has `score: number | null`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Frontend handles missing score, target and work model

**Requirement:** REQ-002, REQ-003
**Files:**
- Modify: `src/features/filter-board/model/filter.ts`
- Modify: `src/widgets/board/model/group.ts`
- Modify: `src/entities/job/ui/JobCard.tsx`
- Modify: `src/entities/job/ui/JobDialog.tsx`
- Modify: `src/features/search-jobs/ui/JobCommand.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/features/filter-board/model/filter.test.ts`
- Test: `src/widgets/board/model/group.test.ts`
- Test: `src/entities/job/ui/JobCard.test.tsx`

**Depends on:** Task 1.

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Add to `src/features/filter-board/model/filter.test.ts` (create the `describe` if needed):

```ts
describe('filterCards', () => {
  it('keeps a card without a score at any minimum', () => {
    const cards = [makeCard({ id: 'a', score: null }), makeCard({ id: 'b', score: 50 })];
    expect(filterCards(cards, DEFAULT_FILTERS).map(c => c.id)).toEqual(['a']);
  });

  it('hides a card without a target only when a target is selected', () => {
    const card = makeCard({ target: null });
    expect(filterCards([card], EMPTY_FILTERS)).toHaveLength(1);
    expect(filterCards([card], { ...EMPTY_FILTERS, targets: ['data'] })).toHaveLength(0);
  });
});
```

Add to `src/widgets/board/model/group.test.ts`:

```ts
it('groupByStage sorts the inbox by score then newest', () => {
  const found = (at: string) => [{ at, kind: 'found' as const, label: 'x' }];
  const cards = [
    makeCard({ id: 'none', score: null, timeline: found('2026-10-30') }),
    makeCard({ id: 'old', score: 80, timeline: found('2026-09-01') }),
    makeCard({ id: 'new', score: 80, timeline: found('2026-10-01') }),
    makeCard({ id: 'top', score: 90, timeline: found('2026-08-01') }),
  ];
  expect(groupByStage(cards).inbox.map(c => c.id)).toEqual(['top', 'new', 'old', 'none']);
});

it('keeps the store order in the other columns', () => {
  const cards = [makeCard({ id: 'a', stage: 'triage', score: 10 }), makeCard({ id: 'b', stage: 'triage', score: 90 })];
  expect(groupByStage(cards).triage.map(c => c.id)).toEqual(['a', 'b']);
});
```

Add to `src/entities/job/ui/JobCard.test.tsx` inside its `describe('JobCard', ...)`:

```tsx
it('shows no score label when the score is null', () => {
  render(<JobCard card={makeCard({ score: null, target: null, workModel: null })} onOpen={() => {}} />);
  expect(screen.getByText('Sem nota')).toBeInTheDocument();
  expect(screen.queryByText(/undefined|null/)).not.toBeInTheDocument();
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/features/filter-board src/widgets/board/model src/entities/job/ui/JobCard --reporter=verbose 2>&1 | tail -15`
Expected: FAIL, "keeps a card without a score" returns `[]` and the JobCard test finds no "Sem nota"

**Step 3: Write minimal implementation**

`filter.ts`, inside `filterCards`:

```ts
      (filters.targets.length === 0 || (card.target !== null && filters.targets.includes(card.target))) &&
      // A job without a score is unjudged, not bad: a minimum must not hide it.
      (card.score === null || card.score >= filters.minScore) &&
```

`group.ts`:

```ts
import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

const foundAt = (card: BoardCard) => card.timeline.find((event) => event.kind === "found")?.at ?? "";

function byScoreThenNewest(a: BoardCard, b: BoardCard): number {
  if (a.score !== b.score) return (b.score ?? -1) - (a.score ?? -1);
  return foundAt(b).localeCompare(foundAt(a));
}

export function groupByStage(cards: BoardCard[]): Record<Stage, BoardCard[]> {
  const groups = Object.fromEntries(STAGES.map((stage) => [stage, [] as BoardCard[]])) as Record<
    Stage,
    BoardCard[]
  >;
  for (const card of cards) groups[card.stage].push(card);
  groups.inbox.sort(byScoreThenNewest);
  return groups;
}
```

`JobCard.tsx`, the score badge:

```tsx
        {card.score === null ? (
          <span className="rounded-md border border-noir-border px-1.5 py-0.5 text-xs text-noir-muted">
            {t('card.noScore')}
          </span>
        ) : (
          <span className={cn('rounded-md border px-1.5 py-0.5 font-mono text-xs', SCORE_TONE[scoreTone(card.score)])}>
            <span className="sr-only">{`${t('card.scoreLabel')} ${card.score}`}</span>
            <span aria-hidden="true">{card.score}</span>
          </span>
        )}
```

and the badges row:

```tsx
        {card.target && (
          <Badge variant="secondary" className="text-noir-amber">
            {t(`target.${card.target}`)}
          </Badge>
        )}
        {card.workModel && <span className="text-xs text-noir-muted">{t(`workModel.${card.workModel}`)}</span>}
```

`JobDialog.tsx`, the description:

```tsx
          {[
            card.company,
            card.source,
            card.target && t(`target.${card.target}`),
            card.workModel && t(`workModel.${card.workModel}`),
          ]
            .filter(Boolean)
            .join(' · ')}
```

`JobCommand.tsx`:

```tsx
                <span className="font-mono text-xs">{card.score ?? t('card.noScore')}</span>
```

Locales: `card.noScore` is `"Sem nota"` in pt-BR and `"No score"` in en.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose 2>&1 | tail -5 && pnpm typecheck`
Expected: PASS, every test green and no type errors

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: Fictional cards only in tests

**Requirement:** REQ-001
**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/sqlite_store.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, consequences-reviewer

**Step 1: Write the failing test**

In `src-tauri/src/lib.rs` tests, replace `open_store_seeds_only_debug_and_only_when_empty` with:

```rust
    #[test]
    fn open_store_never_seeds() {
        let dir = tempfile::tempdir().unwrap();
        for debug in [false, true] {
            let path = dir.path().join(format!("{debug}.db"));
            let store = open_configured_store(Some(path.as_os_str()), dir.path(), debug).unwrap();
            assert!(store.list_cards().unwrap().is_empty());
        }
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test open_store_never_seeds 2>&1 | tail -5`
Expected: FAIL, the debug database has 17 cards

**Step 3: Write minimal implementation**

`src-tauri/src/lib.rs`: remove the seed from `open_store`, drop its `debug` parameter (update `open_configured_store` and the tests), and gate the data:

```rust
#[cfg(test)]
mod mock_data;
```

```rust
pub fn open_store(path: &Path) -> Result<SqliteStore, String> {
    let fail = |cause: String| format!("Could not open the database at {}:\n{cause}", path.display());
    db_path::prepare(path).map_err(|e| fail(e.to_string()))?;
    SqliteStore::open(path).map_err(|e| fail(e.detail()))
}
```

`open_configured_store` keeps its `debug` parameter: it still picks the directory.

`src-tauri/src/sqlite_store.rs`: add `#[cfg(test)]` to the `impl SqliteStore` block holding `seed_if_empty`.

`src-tauri/src/store.rs`: add `#[cfg(test)]` to `pub struct MockStore`, `impl MockStore` and `impl Store for MockStore`, and remove their now-unused `use std::sync::Mutex` from non-test code by moving it inside a `#[cfg(test)]` import. Update the module comment: `MockStore` backs tests only.

Run `cargo build` to confirm the app compiles without `mock_data`.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo build 2>&1 | tail -1 && cargo test 2>&1 | grep -E "open_store_never_seeds|test result"`
Expected: PASS, `tests::open_store_never_seeds ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: Discard, restore and delete in the store and IPC

**Requirement:** REQ-009, REQ-002
**Files:**
- Modify: `src-tauri/src/domain.rs`
- Modify: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/store_contract.rs`
- Modify: `src-tauri/src/sqlite_store.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Depends on:** Task 3.

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: consequences-reviewer, security-reviewer, business-logic-reviewer, test-reviewer

**Step 1: Write the failing test**

Add to `src-tauri/src/store_contract.rs` and call it from `check_all`:

```rust
fn discards_restores_and_deletes(store: &dyn Store) {
    let total = store.list_cards().unwrap().len();
    store.discard_card("c01", "abaixo do corte de score", "2026-11-02").unwrap();
    assert_eq!(store.list_cards().unwrap().len(), total - 1);
    let discarded = store.list_discarded().unwrap();
    assert_eq!(discarded.len(), 1);
    assert_eq!(discarded[0].id, "c01");
    assert_eq!(discarded[0].reason, "abaixo do corte de score");
    assert_eq!(discarded[0].discarded_at, "2026-11-02");

    assert_eq!(store.restore_card("c01").unwrap().id, "c01");
    assert!(store.list_discarded().unwrap().is_empty());
    assert_eq!(store.list_cards().unwrap().len(), total);
    assert_eq!(store.restore_card("c01"), Err(StoreError::NotFound("c01".into())));

    store.discard_card("c01", "x", "2026-11-02").unwrap();
    store.delete_card("c01", "2026-11-02").unwrap();
    assert!(store.list_discarded().unwrap().is_empty());
    assert_eq!(store.delete_card("c01", "2026-11-02"), Err(StoreError::NotFound("c01".into())));
    assert_eq!(store.card_url("nope"), Err(StoreError::NotFound("nope".into())));
}
```

Add to `src-tauri/src/sqlite_store.rs` tests:

```rust
    #[test]
    fn deleting_keeps_a_tombstone_and_removes_the_timeline() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.seed_if_empty(&crate::mock_data::sample_cards()).unwrap();
        store.discard_card("c01", "x", "2026-11-02").unwrap();
        store.delete_card("c01", "2026-11-02").unwrap();
        let conn = store.lock().unwrap();
        let tombstones: i64 = conn
            .query_row("SELECT COUNT(*) FROM dismissed_jobs WHERE fingerprint = 'c01'", [], |r| r.get(0))
            .unwrap();
        let events: i64 = conn
            .query_row("SELECT COUNT(*) FROM timeline_events WHERE card_id = 'c01'", [], |r| r.get(0))
            .unwrap();
        assert_eq!((tombstones, events), (1, 0));
    }
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test 2>&1 | tail -5`
Expected: FAIL to compile with "no method named `discard_card`"

**Step 3: Write minimal implementation**

`src-tauri/src/domain.rs`:

```rust
/// A job waiting for the user's decision on the Discarded page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiscardedJob {
    pub id: String,
    pub title: String,
    pub company: String,
    pub source: String,
    pub location: Option<String>,
    pub score: Option<u8>,
    pub reason: String,
    /// ISO date, `YYYY-MM-DD`.
    pub discarded_at: String,
    pub job_url: Option<String>,
    pub description: String,
}
```

`src-tauri/src/store.rs`, add to `trait Store`:

```rust
    fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError>;
    fn discard_card(&self, id: &str, reason: &str, at: &str) -> Result<(), StoreError>;
    /// Only a discarded card can be restored; it returns to its stage.
    fn restore_card(&self, id: &str) -> Result<BoardCard, StoreError>;
    /// Removes a discarded card for good and remembers it so it never returns.
    fn delete_card(&self, id: &str, at: &str) -> Result<(), StoreError>;
    /// The stored job link of any card, discarded or not.
    fn card_url(&self, id: &str) -> Result<Option<String>, StoreError>;
```

`MockStore` keeps a second field `discards: Mutex<HashMap<String, (String, String)>>`; `list_cards` filters them out, `list_discarded` builds `DiscardedJob` from the card with an empty description, `delete_card` removes the card and the entry, and each method returns `NotFound` under the same conditions as SQLite.

`src-tauri/src/sqlite_store.rs`:

```rust
const LIST_CARDS: &str =
    concat!("SELECT ", card_columns!(), " FROM cards WHERE discard_reason IS NULL ORDER BY rowid");
const LIST_DISCARDED: &str = "SELECT id, title, company, source, location, score, discard_reason, discarded_at, job_url, description \
     FROM cards WHERE discard_reason IS NOT NULL ORDER BY discarded_at DESC, rowid";
```

`LIST_EVENTS` still reads every event: the map lookup ignores discarded cards, so no change there.

```rust
    fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError> {
        let conn = self.lock()?;
        let jobs = conn
            .prepare_cached(LIST_DISCARDED)?
            .query_map([], |row| {
                Ok(DiscardedJob {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    company: row.get(2)?,
                    source: row.get(3)?,
                    location: row.get(4)?,
                    score: row.get(5)?,
                    reason: row.get(6)?,
                    discarded_at: row.get(7)?,
                    job_url: row.get(8)?,
                    description: row.get(9)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(jobs)
    }

    fn discard_card(&self, id: &str, reason: &str, at: &str) -> Result<(), StoreError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE cards SET discard_reason = ?2, discarded_at = ?3 WHERE id = ?1 AND discard_reason IS NULL",
            params![id, reason, at],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        Ok(())
    }

    fn restore_card(&self, id: &str) -> Result<BoardCard, StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            "UPDATE cards SET discard_reason = NULL, discarded_at = NULL WHERE id = ?1 AND discard_reason IS NOT NULL",
            params![id],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        let card = load_card(&tx, id)?;
        tx.commit()?;
        Ok(card)
    }

    fn delete_card(&self, id: &str, at: &str) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let key: Option<Option<String>> = tx
            .query_row(
                "SELECT url_key FROM cards WHERE id = ?1 AND discard_reason IS NOT NULL",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(key) = key else {
            return Err(StoreError::NotFound(id.to_owned()));
        };
        tx.execute(
            "INSERT OR REPLACE INTO dismissed_jobs (fingerprint, url_key, dismissed_at) VALUES (?1, ?2, ?3)",
            params![id, key, at],
        )?;
        tx.execute("DELETE FROM cards WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    fn card_url(&self, id: &str) -> Result<Option<String>, StoreError> {
        let conn = self.lock()?;
        conn.query_row("SELECT job_url FROM cards WHERE id = ?1", params![id], |row| row.get(0))
            .optional()?
            .ok_or_else(|| StoreError::NotFound(id.to_owned()))
    }
```

The `FOREIGN KEY ... ON DELETE CASCADE` removes the events: `foreign_keys = ON` is set after migrating, and the rebuilt `cards` keeps the reference by name. `load_card` keeps working for discarded ids, which `restore_card` needs after clearing them.

`src-tauri/src/commands.rs`:

```rust
pub fn list_discarded_impl(state: &AppState) -> Result<Vec<DiscardedJob>, String> {
    state.store.list_discarded().map_err(report)
}

pub fn restore_card_impl(state: &AppState, id: &str) -> Result<BoardCard, String> {
    check_id(id)?;
    state.store.restore_card(id).map_err(report)
}

pub fn delete_card_impl(state: &AppState, id: &str, today: &str) -> Result<(), String> {
    check_id(id)?;
    state.store.delete_card(id, today).map_err(report)
}
```

plus `#[tauri::command]` wrappers `list_discarded`, `restore_card`, `delete_card`, registered in `lib.rs`'s `generate_handler!`. Add `src-tauri/src/clock.rs` with `pub fn today() -> String` (UTC civil date from `SystemTime`, days-to-civil algorithm, unit-tested on 1970-01-01 and 2028-02-29) and `pub mod clock;`, so the app needs no date crate. Extend `every_command_logs_the_storage_detail_and_hides_it` and `FailingStore` with the new methods.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test 2>&1 | grep -E "contract|tombstone|test result"`
Expected: PASS, both contract tests and `deleting_keeps_a_tombstone_and_removes_the_timeline ... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: URL key and target from the title

**Requirement:** REQ-006, REQ-007
**Files:**
- Create: `src-tauri/src/identity.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, test-reviewer, type-safety-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/identity.rs` with the tests. The expected keys were produced by `job_radar.identity.url_key` in the MIT repo `nandomoreirame/job-radar`; regenerate them with `PYTHONPATH=src python3 -c "from job_radar.identity import url_key; print(url_key('...'))"` from `../job-radar`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Target;

    #[test]
    fn url_key_matches_the_crawler() {
        for (url, key) in [
            ("http://WWW.Example.com/jobs/1/", "https://example.com/jobs/1"),
            ("https://example.com/a?utm_source=x&b=2&a=1#frag", "https://example.com/a?a=1&b=2"),
            ("https://example.com:8080/a?gclid=1", "https://example.com:8080/a"),
            ("https://boards.greenhouse.io/x/jobs/1?gh_src=abc", "https://boards.greenhouse.io/x/jobs/1"),
            ("https://example.com/a?gh_src=keep", "https://example.com/a?gh_src=keep"),
            ("https://example.com/a?q=caf%C3%A9+bar", "https://example.com/a?q=caf%C3%A9+bar"),
            ("not a url", ""),
            ("", ""),
        ] {
            assert_eq!(url_key(url), key, "{url}");
        }
    }

    #[test]
    fn target_comes_from_the_title_family() {
        for (title, target) in [
            ("Engenheiro de IA Sênior", Some(Target::AiLlm)),
            ("Data Engineer", Some(Target::Data)),
            ("Tech Lead Backend", Some(Target::TechLead)),
            ("Desenvolvedor Full Stack", Some(Target::FullStack)),
            ("AI Tech Lead", Some(Target::AiLlm)),
            ("Analista de Sistemas", None),
        ] {
            assert_eq!(target_from_title(title), target, "{title}");
        }
    }
}
```

Add `pub mod identity;` to `lib.rs`.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test identity 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find function `url_key` in this scope"

**Step 3: Write minimal implementation**

`Cargo.toml`: `regex = "1"` and `url = "2"`. Prepend to `identity.rs`:

```rust
//! Job identity shared with the crawler. Must stay equal to `job_radar/identity.py`:
//! a stored key that drifts would let a known job back in.

use std::sync::LazyLock;

use regex::Regex;
use url::Url;

use crate::domain::Target;

const TRACKING: [&str; 2] = ["gclid", "fbclid"];
const HOST_TRACKING: [(&str, &[&str]); 4] = [
    ("linkedin.com", &["trk", "trackingid", "refid", "ebp", "lipi", "originalsubdomain"]),
    ("greenhouse.io", &["gh_src"]),
    ("lever.co", &["lever-source", "lever-origin"]),
    ("ashbyhq.com", &["ashby_src"]),
];

/// https forced, host lowercased without `www.`, no fragment or trailing slash,
/// no tracking params, the others sorted. Invalid input gives an empty key.
pub fn url_key(raw: &str) -> String {
    let Ok(parsed) = Url::parse(raw.trim()) else { return String::new() };
    if !matches!(parsed.scheme(), "http" | "https") {
        return String::new();
    }
    let Some(host) = parsed.host_str() else { return String::new() };
    let host = host.to_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_owned();
    let netloc = match parsed.port() {
        Some(port) if port != 80 && port != 443 => format!("{host}:{port}"),
        _ => host.clone(),
    };
    let path = parsed.path();
    let path = if path == "/" { "" } else { path.trim_end_matches('/') };
    let host_params = HOST_TRACKING
        .iter()
        .find(|(suffix, _)| host == *suffix || host.ends_with(&format!(".{suffix}")))
        .map_or(&[][..], |(_, params)| *params);
    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(key, _)| {
            let lower = key.to_lowercase();
            !lower.starts_with("utm_") && !TRACKING.contains(&lower.as_str()) && !host_params.contains(&lower.as_str())
        })
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    pairs.sort_by(|a, b| (a.0.to_lowercase(), &a.1).cmp(&(b.0.to_lowercase(), &b.1)));
    let query = url::form_urlencoded::Serializer::new(String::new()).extend_pairs(&pairs).finish();
    let mut key = format!("https://{netloc}{path}");
    if !query.is_empty() {
        key.push('?');
        key.push_str(&query);
    }
    key
}

static TITLE_FAMILIES: LazyLock<Vec<(Regex, Target)>> = LazyLock::new(|| {
    [
        (r"(?i)\b(ai|i\.?a\.?|intelig[êe]ncia artificial|llm|genai|machine learning|ml)\b", Target::AiLlm),
        (r"(?i)\b(dados|data)\b", Target::Data),
        (r"(?i)\b(tech lead|l[íi]der t[ée]cnico|staff|principal|arquitet\w*)\b", Target::TechLead),
        (r"(?i)\b(full ?stack|back ?end|software)\b", Target::FullStack),
    ]
    .into_iter()
    .filter_map(|(pattern, target)| Regex::new(pattern).ok().map(|regex| (regex, target)))
    .collect()
});

/// Same title families as the crawler's score, heaviest first.
pub fn target_from_title(title: &str) -> Option<Target> {
    TITLE_FAMILIES.iter().find(|(regex, _)| regex.is_match(title)).map(|(_, target)| *target)
}
```

If a case from Python differs (query encoding is the likely one), fix the Rust side; the Python key is the reference. `crawler-core` Task 1 adds generated vectors over this function.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test identity 2>&1 | grep -E "identity::tests|test result"`
Expected: PASS, both tests `... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: Import seen.db and candidates.json

**Requirement:** REQ-004, REQ-005, REQ-006, REQ-007
**Files:**
- Create: `src-tauri/src/import.rs`
- Modify: `src-tauri/src/details.rs`
- Modify: `src-tauri/src/sqlite_store.rs`
- Modify: `src-tauri/src/lib.rs`

**Depends on:** Tasks 4 and 5.

**Team:**
- Track: data
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, consequences-reviewer, test-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/import.rs` with the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Stage, Target, WorkModel};
    use crate::store::Store;
    use rusqlite::{Connection, params};
    use std::path::{Path, PathBuf};

    const FP1: &str = "aaaaaaaaaaaaaaaa";
    const FP2: &str = "bbbbbbbbbbbbbbbb";
    const FP3: &str = "cccccccccccccccc";

    fn seen_db(dir: &Path, rows: &[(&str, &str, &str)]) -> PathBuf {
        let path = dir.join("seen.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE seen (fingerprint TEXT PRIMARY KEY, source TEXT, title TEXT, company TEXT, url TEXT, sent_at TEXT, url_key TEXT, simhash TEXT)",
        )
        .unwrap();
        for (fp, title, url) in rows {
            conn.execute(
                "INSERT INTO seen (fingerprint, source, title, company, url, sent_at) VALUES (?1, 'gupy', ?2, 'Acme', ?3, '2026-10-05T10:00:00.123')",
                params![fp, title, url],
            )
            .unwrap();
        }
        path
    }

    fn candidates(dir: &Path, items: serde_json::Value) -> PathBuf {
        let path = dir.join("candidates.json");
        std::fs::write(&path, items.to_string()).unwrap();
        path
    }

    fn candidate(fp: &str, url: &str) -> serde_json::Value {
        serde_json::json!({
            "fingerprint": fp, "source": "greenhouse", "title": "Data Engineer", "company": "Beta",
            "url": url, "description": "Python e AWS", "location": "Remoto",
            "work_mode_hint": "remote", "published_at": "2026-10-20T08:00:00+00:00",
            "contact_email": "rh@beta.example", "contact_phone": "", "tags": [], "other_locations": [], "external_id": "1"
        })
    }

    fn store() -> crate::sqlite_store::SqliteStore {
        crate::sqlite_store::SqliteStore::open_in_memory().unwrap()
    }

    #[test]
    fn routes_seen_to_inbox_and_candidates_to_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let cand = candidates(dir.path(), serde_json::json!([candidate(FP2, "https://a.example/2")]));
        let store = store();
        let report = import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        assert_eq!((report.inbox, report.discarded, report.duplicates, report.dismissed, report.invalid), (1, 1, 0, 0, 0));
        let cards = store.list_cards().unwrap();
        assert_eq!((cards[0].id.as_str(), cards[0].stage, cards[0].unread), (FP1, Stage::Inbox, true));
        let discarded = store.list_discarded().unwrap();
        assert_eq!((discarded[0].id.as_str(), discarded[0].reason.as_str()), (FP2, BELOW_CUT));
    }

    #[test]
    fn maps_a_record_to_a_card() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Engenheiro de IA", "https://a.example/1")]);
        let cand = candidates(dir.path(), serde_json::json!([candidate(FP2, "https://a.example/2")]));
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        let card = store.list_cards().unwrap().remove(0);
        assert_eq!(card.score, None);
        assert_eq!(card.target, Some(Target::AiLlm));
        assert_eq!(card.work_model, None);
        assert_eq!(card.job_url.as_deref(), Some("https://a.example/1"));
        assert_eq!(card.timeline[0].at, "2026-10-05");
        let job = store.restore_card(FP2).unwrap();
        assert_eq!(job.work_model, Some(WorkModel::Remote));
        assert_eq!(job.recruiter_email.as_deref(), Some("rh@beta.example"));
        assert_eq!(job.location.as_deref(), Some("Remoto"));
        assert_eq!(job.timeline[0].at, "2026-10-20");
    }

    #[test]
    fn importing_twice_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let store = store();
        import_history(&store, &seen, None, "2026-11-02").unwrap();
        store.move_card(FP1, Stage::Triage, None).unwrap();
        let details = crate::domain::CardDetails { notes: "minha nota".into(), ..Default::default() };
        store.update_card(FP1, details).unwrap();
        let before = store.list_cards().unwrap();
        let report = import_history(&store, &seen, None, "2026-11-03").unwrap();
        assert_eq!((report.inbox, report.duplicates), (0, 1));
        assert_eq!(store.list_cards().unwrap(), before);
    }

    #[test]
    fn seen_wins_over_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let cand = candidates(
            dir.path(),
            serde_json::json!([candidate(FP1, "https://a.example/other"), candidate(FP2, "http://www.a.example/1/")]),
        );
        let store = store();
        let report = import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        assert_eq!((report.inbox, report.discarded, report.duplicates), (1, 0, 2));
        assert!(store.list_discarded().unwrap().is_empty());
    }

    #[test]
    fn a_deleted_job_does_not_return() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP3, "Outra", "https://a.example/3")]);
        let cand = candidates(dir.path(), serde_json::json!([candidate(FP2, "https://a.example/2")]));
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        store.delete_card(FP2, "2026-11-02").unwrap();
        let renamed = candidates(dir.path(), serde_json::json!([candidate(FP1, "https://a.example/2")]));
        let same_id = import_history(&store, &seen, Some(&cand), "2026-11-03").unwrap();
        let same_url = import_history(&store, &seen, Some(&renamed), "2026-11-03").unwrap();
        assert_eq!((same_id.discarded, same_id.dismissed), (0, 1));
        assert_eq!((same_url.discarded, same_url.dismissed), (0, 1));
    }

    #[test]
    fn skips_and_counts_invalid_records() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(
            dir.path(),
            &[
                (FP1, "Boa", "https://a.example/1"),
                (FP2, "Script", "javascript:alert(1)"),
                ("short", "Fingerprint ruim", "https://a.example/3"),
                (FP3, "  ", "https://a.example/4"),
            ],
        );
        let store = store();
        let report = import_history(&store, &seen, None, "2026-11-02").unwrap();
        assert_eq!((report.inbox, report.invalid), (1, 3));
    }

    #[test]
    fn bounds_text_and_drops_an_invalid_email() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[]);
        let mut item = candidate(FP2, "https://a.example/2");
        item["title"] = serde_json::json!("x".repeat(500));
        item["contact_email"] = serde_json::json!("not an email");
        let cand = candidates(dir.path(), serde_json::json!([item]));
        let store = store();
        import_history(&store, &seen, Some(&cand), "2026-11-02").unwrap();
        let job = store.restore_card(FP2).unwrap();
        assert_eq!(job.title.chars().count(), 200);
        assert_eq!(job.recruiter_email, None);
    }

    #[test]
    fn unreadable_candidates_change_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let seen = seen_db(dir.path(), &[(FP1, "Tech Lead", "https://a.example/1")]);
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, "{ not json").unwrap();
        let store = store();
        assert!(import_history(&store, &seen, Some(&bad), "2026-11-02").is_err());
        assert!(import_history(&store, &dir.path().join("missing.db"), None, "2026-11-02").is_err());
        assert!(store.list_cards().unwrap().is_empty());
    }
}
```

Add `pub mod import;` to `lib.rs`.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test import 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find function `import_history` in this scope"

**Step 3: Write minimal implementation**

`src-tauri/src/details.rs`: expose the existing checks, without changing them:

```rust
pub fn is_http_url(value: &str) -> bool {
    url("", Some(value.to_owned())).is_ok_and(|v| v.is_some())
}

pub fn is_email(value: &str) -> bool {
    email("", Some(value.to_owned())).is_ok_and(|v| v.is_some())
}
```

Prepend to `src-tauri/src/import.rs`:

```rust
//! One-way load of the crawler's history. Both files are untrusted: records are
//! validated one by one, and the whole import commits or none of it does.

use std::collections::HashSet;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;

use crate::details::{is_email, is_http_url};
use crate::domain::{Artifacts, BoardCard, EventKind, Stage, TimelineEvent, WorkModel};
use crate::identity::{target_from_title, url_key};
use crate::sqlite_store::SqliteStore;

pub const BELOW_CUT: &str = "abaixo do corte de score";
const MAX_TEXT: usize = 200;
const MAX_DESCRIPTION: usize = 50_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub inbox: usize,
    pub discarded: usize,
    pub duplicates: usize,
    pub dismissed: usize,
    pub invalid: usize,
}

/// A validated job ready to store. `discard` is the reason on the Discarded page.
pub struct NewJob {
    pub card: BoardCard,
    pub url_key: String,
    pub description: String,
    pub discard: Option<(String, String)>,
}

#[derive(Deserialize)]
struct Candidate {
    fingerprint: String,
    source: String,
    title: String,
    company: String,
    url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    location: String,
    work_mode_hint: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    contact_email: String,
}

struct Seen {
    fingerprint: String,
    source: String,
    title: String,
    company: String,
    url: String,
    sent_at: String,
}

fn bounded(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

fn is_fingerprint(value: &str) -> bool {
    value.len() == 16 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn date_or(raw: &str, today: &str) -> String {
    let date = raw.get(..10).unwrap_or("");
    let shape = date.len() == 10 && date.as_bytes()[4] == b'-' && date.as_bytes()[7] == b'-';
    if shape { date.to_owned() } else { today.to_owned() }
}

fn work_model(hint: Option<&str>) -> Option<WorkModel> {
    match hint?.trim().to_lowercase().replace(['-', '_'], "").as_str() {
        "remote" => Some(WorkModel::Remote),
        "hybrid" => Some(WorkModel::Hybrid),
        "onsite" | "presencial" => Some(WorkModel::Onsite),
        _ => None,
    }
}

fn base_card(id: &str, source: &str, title: &str, company: &str, url: &str, found: &str) -> BoardCard {
    let source = bounded(source, MAX_TEXT);
    BoardCard {
        id: id.to_owned(),
        title: bounded(title, MAX_TEXT),
        company: bounded(company, MAX_TEXT),
        target: target_from_title(title),
        work_model: None,
        location: None,
        score: None,
        stage: Stage::Inbox,
        outcome: None,
        process_step: None,
        salary: None,
        artifacts: Artifacts { cv: false, coverage: false, interview_prep: false },
        last_activity: None,
        next_action: None,
        ai_running: None,
        unread: true,
        timeline: vec![TimelineEvent {
            at: found.to_owned(),
            kind: EventKind::Found,
            label: format!("Encontrada em {source}"),
        }],
        next_action_date: None,
        recruiter_name: None,
        recruiter_email: None,
        recruiter_linkedin: None,
        job_url: Some(url.to_owned()),
        application_url: None,
        notes: String::new(),
        source,
    }
}

fn valid_record(fingerprint: &str, title: &str, url: &str) -> bool {
    is_fingerprint(fingerprint) && !title.trim().is_empty() && is_http_url(url)
}

fn read_seen(path: &Path) -> Result<Vec<Seen>, String> {
    let read = || -> rusqlite::Result<Vec<Seen>> {
        // Flags keep a wrong path from creating an empty database.
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut statement =
            conn.prepare("SELECT fingerprint, source, title, company, url, sent_at FROM seen ORDER BY sent_at")?;
        let rows = statement.query_map([], |row| {
            Ok(Seen {
                fingerprint: row.get(0)?,
                source: row.get(1)?,
                title: row.get(2)?,
                company: row.get(3)?,
                url: row.get(4)?,
                sent_at: row.get(5)?,
            })
        })?;
        rows.collect()
    };
    read().map_err(|e| format!("{}: {e}", path.display()))
}

fn read_candidates(path: &Path) -> Result<Vec<Candidate>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn import_history(
    store: &SqliteStore,
    seen: &Path,
    candidates: Option<&Path>,
    today: &str,
) -> Result<ImportReport, String> {
    // Read both files before writing, so a bad file changes nothing.
    let seen = read_seen(seen)?;
    let candidates = candidates.map(read_candidates).transpose()?.unwrap_or_default();

    let mut report = ImportReport::default();
    let mut jobs = Vec::with_capacity(seen.len() + candidates.len());
    for row in seen {
        if !valid_record(&row.fingerprint, &row.title, &row.url) {
            report.invalid += 1;
            continue;
        }
        let found = date_or(&row.sent_at, today);
        jobs.push(NewJob {
            card: base_card(&row.fingerprint, &row.source, &row.title, &row.company, &row.url, &found),
            url_key: url_key(&row.url),
            description: String::new(),
            discard: None,
        });
    }
    for item in candidates {
        if !valid_record(&item.fingerprint, &item.title, &item.url) {
            report.invalid += 1;
            continue;
        }
        let found = date_or(item.published_at.as_deref().unwrap_or(""), today);
        let mut card = base_card(&item.fingerprint, &item.source, &item.title, &item.company, &item.url, &found);
        card.work_model = work_model(item.work_mode_hint.as_deref());
        card.location = Some(bounded(&item.location, MAX_TEXT)).filter(|v| !v.is_empty());
        card.recruiter_email = Some(item.contact_email.trim().to_owned()).filter(|v| is_email(v));
        jobs.push(NewJob {
            card,
            url_key: url_key(&item.url),
            description: bounded(&item.description, MAX_DESCRIPTION),
            discard: Some((BELOW_CUT.to_owned(), today.to_owned())),
        });
    }
    store.insert_new_jobs(jobs, &mut report).map_err(|e| e.detail())?;
    Ok(report)
}
```

`src-tauri/src/sqlite_store.rs`, a new inherent method. It reads the known keys once and checks them in memory, so there is no query per record:

```rust
impl SqliteStore {
    /// Inserts the jobs not seen before, in one transaction, and fills the counts.
    pub fn insert_new_jobs(&self, jobs: Vec<NewJob>, report: &mut ImportReport) -> Result<(), StoreError> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let keys = |sql: &str| -> rusqlite::Result<(HashSet<String>, HashSet<String>)> {
            let mut statement = tx.prepare(sql)?;
            let rows = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)))?;
            let mut ids = HashSet::new();
            let mut urls = HashSet::new();
            for row in rows {
                let (id, url) = row?;
                ids.insert(id);
                urls.extend(url);
            }
            Ok((ids, urls))
        };
        let (mut known_ids, mut known_urls) = keys("SELECT id, url_key FROM cards")?;
        let (dismissed_ids, dismissed_urls) = keys("SELECT fingerprint, url_key FROM dismissed_jobs")?;
        for job in jobs {
            let url_key = Some(job.url_key.as_str()).filter(|key| !key.is_empty());
            if dismissed_ids.contains(&job.card.id) || url_key.is_some_and(|key| dismissed_urls.contains(key)) {
                report.dismissed += 1;
                continue;
            }
            if known_ids.contains(&job.card.id) || url_key.is_some_and(|key| known_urls.contains(key)) {
                report.duplicates += 1;
                continue;
            }
            insert_card(&tx, &job.card)?;
            let (reason, at) = job.discard.unzip();
            tx.execute(
                "UPDATE cards SET url_key = ?2, description = ?3, discard_reason = ?4, discarded_at = ?5 WHERE id = ?1",
                params![job.card.id, url_key, job.description, reason, at],
            )?;
            known_ids.insert(job.card.id.clone());
            if let Some(key) = url_key {
                known_urls.insert(key.to_owned());
            }
            if reason.is_some() {
                report.discarded += 1;
            } else {
                report.inbox += 1;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
```

`Option::unzip` works on `Option<(A, B)>`. Add `use crate::import::{ImportReport, NewJob};` and `use std::collections::HashSet;`. `insert_card` loses its `#[cfg(test)]` link if Task 3 added any.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test import 2>&1 | grep -E "import::tests|test result"`
Expected: PASS, the eight `import::tests::... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: The import subcommand

**Requirement:** REQ-004
**Files:**
- Create: `src-tauri/src/cli.rs`
- Modify: `src-tauri/src/lib.rs`

**Depends on:** Task 6.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, test-reviewer

**Step 1: Write the failing test**

Create `src-tauri/src/cli.rs` with the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_the_import_command() {
        assert_eq!(parse(&args(&[])), Ok(None));
        assert_eq!(
            parse(&args(&["import", "--seen", "a.db", "--candidates", "b.json"])),
            Ok(Some(ImportArgs { seen: "a.db".into(), candidates: Some("b.json".into()) }))
        );
        assert_eq!(
            parse(&args(&["import", "--seen", "a.db"])),
            Ok(Some(ImportArgs { seen: "a.db".into(), candidates: None }))
        );
        for bad in [&["import"][..], &["import", "--seen"], &["import", "--nope", "x"], &["other"]] {
            assert!(parse(&args(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn reports_every_count() {
        let report = ImportReport { inbox: 1, discarded: 2, duplicates: 3, dismissed: 4, invalid: 5 };
        assert_eq!(format_report(&report), "inbox: 1\ndiscarded: 2\nduplicates: 3\ndismissed: 4\ninvalid: 5");
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test cli 2>&1 | tail -5`
Expected: FAIL to compile with "cannot find function `parse` in this scope"

**Step 3: Write minimal implementation**

Prepend to `src-tauri/src/cli.rs`:

```rust
//! Command line of the app binary: no arguments opens the window.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::import::ImportReport;

pub const USAGE: &str = "usage: job-radar-desktop import --seen <seen.db> [--candidates <candidates.json>]";

#[derive(Debug, PartialEq, Eq)]
pub struct ImportArgs {
    pub seen: PathBuf,
    pub candidates: Option<PathBuf>,
}

pub fn parse(args: &[OsString]) -> Result<Option<ImportArgs>, String> {
    let Some((command, rest)) = args.split_first() else { return Ok(None) };
    if command != "import" {
        return Err(USAGE.to_owned());
    }
    let mut seen = None;
    let mut candidates = None;
    let mut rest = rest.iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or(USAGE)?;
        match flag.to_str() {
            Some("--seen") => seen = Some(PathBuf::from(value)),
            Some("--candidates") => candidates = Some(PathBuf::from(value)),
            _ => return Err(USAGE.to_owned()),
        }
    }
    let seen = seen.ok_or(USAGE)?;
    Ok(Some(ImportArgs { seen, candidates }))
}

pub fn format_report(report: &ImportReport) -> String {
    format!(
        "inbox: {}\ndiscarded: {}\nduplicates: {}\ndismissed: {}\ninvalid: {}",
        report.inbox, report.discarded, report.duplicates, report.dismissed, report.invalid
    )
}
```

`src-tauri/src/lib.rs`: add `pub mod cli;`, and at the top of `run()`:

```rust
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match cli::parse(&args) {
        Ok(None) => {}
        Ok(Some(import)) => std::process::exit(run_import(&import)),
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(1);
        }
    }
```

```rust
/// Runs without Tauri or GTK: this is a terminal command.
fn run_import(args: &cli::ImportArgs) -> i32 {
    let result = dirs::data_dir()
        .ok_or_else(|| "Could not determine the user's data directory".to_owned())
        .and_then(|data_dir| {
            let env_override = std::env::var_os(db_path::ENV_VAR);
            let store = open_configured_store(env_override.as_deref(), &data_dir, cfg!(debug_assertions))?;
            import::import_history(&store, &args.seen, args.candidates.as_deref(), &clock::today())
        });
    match result {
        Ok(report) => {
            println!("{}", cli::format_report(&report));
            0
        }
        Err(cause) => {
            eprintln!("job-radar-desktop: {cause}");
            1
        }
    }
}
```

Tauri dev passes its own arguments only through the `--` separator, so a plain `pnpm tauri dev` has none and opens the window.

Manual check, on fictional data under a scratch path: `JOB_RADAR_DB=<scratch>/x.db cargo run -- import --seen <scratch>/seen.db` prints the counts, opens no window and exits 0; a missing file exits 1 with the cause.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test cli 2>&1 | grep -E "cli::tests|test result"`
Expected: PASS, both `cli::tests::... ok`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 8: Open a job's link by id

**Requirement:** REQ-010
**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/shared/api/board.ts`
- Modify: `src/test/factories.ts`
- Modify: `src/entities/job/ui/JobDialog.tsx`
- Modify: `src/pages/board/BoardPage.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/entities/job/ui/JobDialog.test.tsx`

**Depends on:** Task 4.

**Team:**
- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: security-reviewer, business-logic-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

`src-tauri/src/commands.rs` tests:

```rust
    #[test]
    fn opens_only_a_stored_http_url() {
        let state = state();
        let details = CardDetails { job_url: Some("https://a.example/1".into()), ..CardDetails::default() };
        update_card_impl(&state, "c01", details).unwrap();
        let opened = std::cell::RefCell::new(Vec::new());
        let open = |url: &str| {
            opened.borrow_mut().push(url.to_owned());
            Ok(())
        };
        open_job_url_impl(&state, "c01", open).unwrap();
        assert_eq!(*opened.borrow(), ["https://a.example/1"]);

        let cleared = state.store.update_card("c01", CardDetails::default()).unwrap();
        assert_eq!(cleared.job_url, None);
        assert_eq!(open_job_url_impl(&state, "c01", open), Err("no_url".to_string()));
        // A value that bypassed validation must not reach the OS.
        let bad = CardDetails { job_url: Some("javascript:alert(1)".into()), ..CardDetails::default() };
        state.store.update_card("c01", bad).unwrap();
        assert_eq!(open_job_url_impl(&state, "c01", open), Err("invalid_url".to_string()));
        assert_eq!(open_job_url_impl(&state, "", open), Err("invalid card id".to_string()));
        assert_eq!(opened.borrow().len(), 1);
    }
```

`src/entities/job/ui/JobDialog.test.tsx`:

```tsx
  it('opens the job link by id', async () => {
    const onOpenLink = vi.fn();
    render(
      <JobDialog card={makeCard({ jobUrl: 'https://a.example/1' })} onOpenChange={() => {}} onSave={saveOk()} onOpenLink={onOpenLink} />,
    );
    await userEvent.click(screen.getByRole('button', { name: 'Abrir vaga' }));
    expect(onOpenLink).toHaveBeenCalledWith('c01');
  });

  it('offers no link without a stored URL', () => {
    render(<JobDialog card={makeCard({ jobUrl: null })} onOpenChange={() => {}} onSave={saveOk()} onOpenLink={vi.fn()} />);
    expect(screen.queryByRole('button', { name: 'Abrir vaga' })).not.toBeInTheDocument();
  });
```

Pass `onOpenLink={vi.fn()}` in the other `JobDialog` renders of this file.

**Step 2: Run test to verify it fails**
Run: `cd src-tauri && cargo test opens_only 2>&1 | tail -3`
Expected: FAIL to compile with "cannot find function `open_job_url_impl`"

**Step 3: Write minimal implementation**

`Cargo.toml`: `tauri-plugin-opener = "2"`.

`src-tauri/src/commands.rs`:

```rust
/// The URL comes from the database by id, re-checked here: the webview can never
/// ask the OS to open a value of its own.
pub fn open_job_url_impl(
    state: &AppState,
    id: &str,
    open: impl Fn(&str) -> Result<(), String>,
) -> Result<(), String> {
    check_id(id)?;
    let url = state.store.card_url(id).map_err(report)?.ok_or("no_url")?;
    if !is_http_url(&url) {
        return Err("invalid_url".into());
    }
    open(&url)
}

#[tauri::command]
pub fn open_job_url(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    open_job_url_impl(&state, &id, |url| {
        app.opener().open_url(url, None::<&str>).map_err(|e| {
            eprintln!("open url: {e}");
            "open_failed".to_owned()
        })
    })
}
```

`lib.rs`: `.plugin(tauri_plugin_opener::init())` on the builder and `commands::open_job_url` in `generate_handler!`. Do not add `opener:*` to `capabilities/default.json`: only Rust uses it.

`src/shared/api/board.ts`: add to `BoardApi` and `boardApi`:

```ts
  /** Opens the card's stored link in the default browser. */
  openLink(id: string): Promise<void>;
```

```ts
  openLink: (id) => invoke<void>("open_job_url", { id }),
```

`src/test/factories.ts`: `openLink: async () => {},` in `fakeApi`.

`JobDialog.tsx`: add `onOpenLink: (id: string) => void` to `Props` and pass it to `JobForm`. In the footer's left group:

```tsx
          {card.jobUrl && (
            <Button type="button" variant="outline" onClick={() => onOpenLink(card.id)}>
              <ExternalLink aria-hidden="true" />
              {t('dialog.openLink')}
            </Button>
          )}
```

`BoardPage.tsx`: `onOpenLink={id => void api.openLink(id)}`. Locales: `dialog.openLink` is `"Abrir vaga"` / `"Open job"`.

**Step 4: Run test to verify it passes**
Run: `cd src-tauri && cargo test opens_only 2>&1 | grep -E "opens_only|test result: " | head -2 && cd .. && pnpm vitest run src/entities/job/ui/JobDialog --reporter=verbose 2>&1 | tail -4`
Expected: PASS, the Rust test `... ok` and the dialog tests green

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 9: Discarded page

**Requirement:** REQ-008, REQ-009, REQ-010
**Files:**
- Create: `src/entities/job/model/use-discarded.ts`
- Create: `src/pages/discarded/DiscardedPage.tsx`
- Test: `src/pages/discarded/DiscardedPage.test.tsx`
- Modify: `src/shared/api/board.ts`
- Modify: `src/shared/config/navigation.ts`
- Modify: `src/widgets/app-shell/Sidebar.tsx`
- Modify: `src/app/App.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`

**Depends on:** Tasks 4 and 8.

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails, security-reviewer

**Step 1: Write the failing test**

Create `src/pages/discarded/DiscardedPage.test.tsx`:

```tsx
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
import { expectNoAxeViolations } from '@/test/axe';
import { makeCard } from '@/test/factories';
import { DiscardedPage } from './DiscardedPage';

const job = (overrides: Partial<DiscardedJob> = {}): DiscardedJob => ({
  id: 'aaaaaaaaaaaaaaaa',
  title: 'Data Engineer',
  company: 'Beta',
  source: 'greenhouse',
  location: 'Remoto',
  score: null,
  reason: 'abaixo do corte de score',
  discardedAt: '2026-11-02',
  jobUrl: 'https://a.example/1',
  description: 'Python e AWS',
  ...overrides,
});

function fakeDiscarded(jobs: DiscardedJob[]): DiscardedApi {
  let state = [...jobs];
  return {
    list: vi.fn(async () => state),
    restore: vi.fn(async (id: string) => {
      state = state.filter(j => j.id !== id);
      return makeCard({ id });
    }),
    remove: vi.fn(async (id: string) => {
      state = state.filter(j => j.id !== id);
    }),
    openLink: vi.fn(async () => {}),
  };
}

const other = job({ id: 'bbbbbbbbbbbbbbbb', title: 'Tech Lead', company: 'Coração Labs', description: '' });

describe('DiscardedPage', () => {
  it('lists and searches discarded jobs', async () => {
    render(<DiscardedPage api={fakeDiscarded([job(), other])} />);
    expect(await screen.findByRole('heading', { name: 'Data Engineer' })).toBeInTheDocument();
    expect(screen.getByText('abaixo do corte de score', { exact: false })).toBeTruthy();
    expect(screen.getByText('Python e AWS')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Buscar descartadas' }), 'coracao');
    expect(screen.queryByRole('heading', { name: 'Data Engineer' })).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Tech Lead' })).toBeInTheDocument();
  });

  it('restores a job', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Mover para a Caixa de entrada' }));
    expect(api.restore).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
    await waitFor(() => expect(screen.queryByRole('article', { name: 'Data Engineer' })).not.toBeInTheDocument());
  });

  it('deletes a job only after confirming', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Excluir' }));
    expect(api.remove).not.toHaveBeenCalled();
    const dialog = await screen.findByRole('dialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancelar' }));
    expect(api.remove).not.toHaveBeenCalled();
    await userEvent.click(within(item).getByRole('button', { name: 'Excluir' }));
    await userEvent.click(within(await screen.findByRole('dialog')).getByRole('button', { name: 'Excluir definitivamente' }));
    expect(api.remove).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
  });

  it('opens the job link by id', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Abrir vaga' }));
    expect(api.openLink).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
  });

  it('shows the empty state', async () => {
    render(<DiscardedPage api={fakeDiscarded([])} />);
    expect(await screen.findByText('Nenhuma vaga descartada.')).toBeInTheDocument();
  });

  it('reports a failed action', async () => {
    const api = fakeDiscarded([job()]);
    vi.mocked(api.restore).mockRejectedValueOnce('storage error');
    render(<DiscardedPage api={api} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Mover para a Caixa de entrada' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível concluir a ação.');
    expect(screen.getByRole('article', { name: 'Data Engineer' })).toBeInTheDocument();
  });

  it('has no axe violations', async () => {
    const { container } = render(<DiscardedPage api={fakeDiscarded([job(), other])} />);
    await screen.findByRole('heading', { name: 'Data Engineer' });
    await expectNoAxeViolations(container);
  });
});
```

Check the helper's exact name in `src/test/axe.ts` before using it.

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/pages/discarded --reporter=verbose 2>&1 | tail -5`
Expected: FAIL, "Failed to resolve import ./DiscardedPage"

**Step 3: Write minimal implementation**

`src/shared/api/board.ts`:

```ts
import type { DiscardedJob } from "@/shared/bindings/DiscardedJob";

export type DiscardedApi = {
  list(): Promise<DiscardedJob[]>;
  restore(id: string): Promise<BoardCard>;
  remove(id: string): Promise<void>;
  openLink(id: string): Promise<void>;
};

export const discardedApi: DiscardedApi = {
  list: () => invoke<DiscardedJob[]>("list_discarded"),
  restore: (id) => invoke<BoardCard>("restore_card", { id }),
  remove: (id) => invoke<void>("delete_card", { id }),
  openLink: (id) => invoke<void>("open_job_url", { id }),
};
```

`src/entities/job/model/use-discarded.ts`:

```ts
import { useCallback, useEffect, useState } from 'react';
import type { DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';

export function useDiscarded(api: DiscardedApi) {
  const [jobs, setJobs] = useState<DiscardedJob[]>([]);
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [actionError, setActionError] = useState(false);

  useEffect(() => {
    let alive = true;
    api.list().then(
      list => {
        if (!alive) return;
        setJobs(list);
        setStatus('ready');
      },
      () => {
        if (alive) setStatus('error');
      },
    );
    return () => {
      alive = false;
    };
  }, [api]);

  const settle = useCallback(async (id: string, action: Promise<unknown>) => {
    try {
      await action;
      setJobs(list => list.filter(job => job.id !== id));
      setActionError(false);
    } catch {
      setActionError(true);
    }
  }, []);

  const restore = useCallback((id: string) => settle(id, api.restore(id)), [api, settle]);
  const remove = useCallback((id: string) => settle(id, api.remove(id)), [api, settle]);
  return { jobs, status, actionError, restore, remove };
}
```

`src/pages/discarded/DiscardedPage.tsx`:

```tsx
import { ExternalLink, RotateCcw, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDiscarded } from '@/entities/job/model/use-discarded';
import { matchesText } from '@/features/filter-board/model/filter';
import { discardedApi, type DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
import { Button } from '@/shared/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/dialog';
import { Input } from '@/shared/ui/input';

type Props = { api?: DiscardedApi };

export function DiscardedPage({ api = discardedApi }: Props) {
  const { t } = useTranslation();
  const { jobs, status, actionError, restore, remove } = useDiscarded(api);
  const [query, setQuery] = useState('');
  const [confirming, setConfirming] = useState<DiscardedJob | null>(null);
  const visible = useMemo(
    () => jobs.filter(job => matchesText(`${job.title} ${job.company} ${job.source}`, query)),
    [jobs, query],
  );

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
      <div className="flex flex-wrap items-center gap-4">
        <h1 className="h-9 text-2xl font-semibold leading-9">{t('discarded.title')}</h1>
        <Input
          type="search"
          aria-label={t('discarded.search')}
          placeholder={t('discarded.search')}
          value={query}
          onChange={event => setQuery(event.target.value)}
          className="h-9 w-72"
        />
      </div>
      {actionError && (
        <p role="alert" className="text-sm text-noir-red">
          {t('discarded.actionError')}
        </p>
      )}
      {status === 'loading' && <p role="status">{t('board.loading')}</p>}
      {status === 'error' && <p role="alert">{t('board.error')}</p>}
      {status === 'ready' && jobs.length === 0 && <p>{t('discarded.empty')}</p>}
      <ul className="flex flex-col gap-3">
        {visible.map(job => (
          <li key={job.id}>
            <article aria-labelledby={`discarded-${job.id}`} className="noir-card flex flex-col gap-2 rounded-lg p-4">
              <h2 id={`discarded-${job.id}`} className="text-sm font-semibold">
                {job.title}
              </h2>
              <p className="text-sm text-noir-muted">
                {[job.company, job.source, job.location].filter(Boolean).join(' · ')}
              </p>
              <p className="text-xs text-noir-muted">
                {job.reason} · <time dateTime={job.discardedAt}>{job.discardedAt}</time>
              </p>
              {job.description && (
                <details>
                  <summary className="cursor-pointer text-xs">{t('discarded.description')}</summary>
                  {/* Plain text on purpose: the description comes from third-party pages. */}
                  <p className="mt-2 max-h-64 overflow-y-auto whitespace-pre-wrap text-sm">{job.description}</p>
                </details>
              )}
              <div className="flex flex-wrap gap-2">
                <Button type="button" variant="outline" size="sm" onClick={() => void restore(job.id)}>
                  <RotateCcw aria-hidden="true" />
                  {t('discarded.restore')}
                </Button>
                {job.jobUrl && (
                  <Button type="button" variant="outline" size="sm" onClick={() => void api.openLink(job.id)}>
                    <ExternalLink aria-hidden="true" />
                    {t('dialog.openLink')}
                  </Button>
                )}
                <Button type="button" variant="outline" size="sm" onClick={() => setConfirming(job)}>
                  <Trash2 aria-hidden="true" />
                  {t('discarded.delete')}
                </Button>
              </div>
            </article>
          </li>
        ))}
      </ul>
      <Dialog open={confirming !== null} onOpenChange={open => !open && setConfirming(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('discarded.confirmTitle')}</DialogTitle>
            <DialogDescription>{t('discarded.confirmBody', { title: confirming?.title })}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setConfirming(null)}>
              {t('discarded.cancel')}
            </Button>
            <Button
              type="button"
              variant="destructive"
              onClick={() => {
                if (confirming) void remove(confirming.id);
                setConfirming(null);
              }}
            >
              {t('discarded.confirmDelete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
```

`navigation.ts`: `["board", "discarded", "profile", "resumes", "prep", "settings"]`.

`Sidebar.tsx`: `{ id: 'discarded', Icon: Archive }` right after `board`, importing `Archive` from `lucide-react`.

`App.tsx`:

```tsx
      renderPage={(page) => {
        if (page === "board") return <BoardPage />;
        if (page === "discarded") return <DiscardedPage />;
        return <PlaceholderPage page={page} />;
      }}
```

Each page remounts on navigation, so a restored job shows on the board with no extra wiring.

Rust side: `delete_card` command computes the date with `crate::clock::today()`.

Locales (`pt-BR` / `en`): `nav.discarded` "Descartadas" / "Discarded"; `discarded.title` "Vagas descartadas" / "Discarded jobs"; `discarded.search` "Buscar descartadas" / "Search discarded"; `discarded.empty` "Nenhuma vaga descartada." / "No discarded jobs."; `discarded.description` "Ver descrição" / "Show description"; `discarded.restore` "Mover para a Caixa de entrada" / "Move to Inbox"; `discarded.delete` "Excluir" / "Delete"; `discarded.confirmTitle` "Excluir esta vaga?" / "Delete this job?"; `discarded.confirmBody` "{{title}} sai daqui e não volta em nenhuma busca. Não dá para desfazer." / "{{title}} leaves this list and never returns in a search. This cannot be undone."; `discarded.cancel` "Cancelar" / "Cancel"; `discarded.confirmDelete` "Excluir definitivamente" / "Delete for good"; `discarded.actionError` "Não foi possível concluir a ação." / "The action could not be completed."

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose 2>&1 | tail -6 && pnpm typecheck`
Expected: PASS, the seven `DiscardedPage` tests and the whole suite green

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 10: Empty board state

**Requirement:** REQ-011
**Files:**
- Modify: `src/pages/board/BoardPage.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/pages/board/BoardPage.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

```tsx
  it('shows the empty state with no cards', async () => {
    render(<BoardPage api={fakeApi([])} />);
    expect(await screen.findByText('Nenhuma vaga ainda')).toBeInTheDocument();
    expect(screen.getByText(/job-radar-desktop import/)).toBeInTheDocument();
  });

  it('keeps the board when only the filters hide the cards', async () => {
    render(<BoardPage api={fakeApi([makeCard({ score: 10 })])} />);
    await screen.findByRole('heading', { name: 'Caixa de entrada', exact: false });
    expect(screen.queryByText('Nenhuma vaga ainda')).not.toBeInTheDocument();
  });
```

Adjust the column heading query to what `BoardColumn` renders in the existing tests of this file.

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/pages/board/BoardPage.test --reporter=verbose 2>&1 | tail -6`
Expected: FAIL, unable to find the text "Nenhuma vaga ainda"

**Step 3: Write minimal implementation**

`BoardPage.tsx`, replacing the `status === 'ready'` block:

```tsx
      {status === 'ready' && cards.length === 0 && (
        <div className="px-6">
          <h2 className="text-lg font-semibold">{t('board.empty.title')}</h2>
          <p className="text-sm text-noir-muted">{t('board.empty.body')}</p>
          <code className="mt-2 block font-mono text-xs">
            job-radar-desktop import --seen seen.db --candidates candidates.json
          </code>
        </div>
      )}
      {status === 'ready' && cards.length > 0 && (
        <Board
          cards={visible}
          onOpen={card => setOpenId(card.id)}
          onMove={(id, stage, outcome) => void move(id, stage, outcome)}
        />
      )}
```

Locales: `board.empty.title` "Nenhuma vaga ainda" / "No jobs yet"; `board.empty.body` "Importe o histórico do crawler pelo terminal:" / "Import the crawler's history from the terminal:".

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run src/pages/board --reporter=verbose 2>&1 | tail -5`
Expected: PASS, including the a11y test of the board page

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 11: README, formatting and the real import

**Requirement:** REQ-012
**Files:**
- Modify: `README.md`
- Modify: `README.pt-br.md`

**Team:**
- Track: docs
- Implementer: rpi-implementer (skills: readme-writer)
- Reviewers: self-review

**Step 1: Write the failing test**

```bash
grep -c "development build starts with fictional jobs\|desenvolvimento começa com vagas fictícias" README.md README.pt-br.md
```

**Step 2: Run test to verify it fails**
Run: `grep -c "job-radar-desktop import" README.md README.pt-br.md`
Expected: FAIL (the docs are stale): both counts `0`

**Step 3: Write minimal implementation**

`README.md`:
- Quick start sentence: `A frameless, maximized window opens with an empty board; see Usage to load jobs.`
- New rows in the Usage table: `Load a crawler's history` | `` `job-radar-desktop import --seen seen.db --candidates candidates.json` `` (safe to repeat) and `Review discarded jobs` | `Open Discarded in the sidebar, then move a job to the Inbox or delete it for good`.
- Usage note: a job from the import has no score until the crawler reads its page.
- Security, replacing the "Minimal permissions" line ending: add `The opener plugin is used only from Rust, for a link already stored and checked; the webview cannot open a URL of its own.`
- Security, new bullet: `**Import reads local files only,** read-only, and treats their content as untrusted.`
- The "Status" lines keep saying there is no crawler yet.

`README.pt-br.md`: the same edits in Portuguese, with full accents.

Then run `python3 ~/.claude/skills/readme-writer/scripts/check_readme.py README.md` and the same for the pt-BR file; fix every FAIL. Run the gates: `bash ~/.claude/hooks/helpers/rpi-gates.sh tier1 --slug real-jobs-import .`, then `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` from `src-tauri/`.

Manual check, recorded in `docs/specs/real-jobs-import/STATE.md`. Ask the user before deleting the debug database with the fictional cards:
1. Copy `seen.db` and `candidates.json` from the server to a scratch folder outside the repo.
2. Run the import from `src-tauri/`: `cargo run -- import --seen <scratch>/seen.db --candidates <scratch>/candidates.json`. Expect `inbox: 207`, `discarded: 137`.
3. `pnpm tauri dev`: every job is visible with the default filter, marked "Sem nota"; drag a card and check the column stays usable with 207 cards.
4. On Descartadas, open a link, restore one job, delete another; re-run the import: `dismissed: 1` and nothing returns.

**Step 4: Run test to verify it passes**
Run: `grep -c "job-radar-desktop import" README.md README.pt-br.md`
Expected: PASS, both counts at least `1`

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
