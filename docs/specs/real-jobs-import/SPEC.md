# Spec: Real Jobs Import

## Problem Statement

The board only ever shows fictional cards: a debug build seeds `sample_cards()` and a release build
starts empty, so nobody can use the app for a real search. The crawler already running on the
author's server recorded the jobs it sent, and the jobs it left below the score cut. Those two files
are the fastest way to a real board, but they lack fields the domain requires today.

## Goals

- No build shows fictional cards; they exist only in tests.
- One command loads the crawler's history into a database, safely and repeatably.
- A job without a score, target or work model is a normal, visible state.
- A Discarded page lets the user curate below-the-cut jobs: read, restore or delete for good.
- A job's link opens in the default browser.

## Context

Decisions taken with the user on 2026-11-02: mocks leave the dev build (reverses `sqlite-store`
REQ-005 and grilling decision 2); all jobs in `seen.db` go to the Inbox; only the jobs in
`candidates.json` go to the Discarded page, with manual delete; the 969 rows of `discarded.tsv` stay
out. The server address never enters this public repo, so import takes file paths.

The scores are recalculated by `crawler-core`; this feature ends with real jobs marked "no score".

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| `seen.db` holds `fingerprint, source, title, company, url, sent_at, url_key, simhash`; no score, target, work model or description | `sqlite3` on the server copy, 2026-11-02, and `job_radar/store.py:25-36` | yes |
| The server copy has 207 rows from 2026-09-22 to 2026-10-31; every fingerprint has 16 hex chars, every url starts with `http`, no `url_key` is empty | `sqlite3` on the server copy, 2026-11-02 | yes |
| `candidates.json` is a list of 137 objects with `company, contact_email, contact_phone, description, external_id, fingerprint, location, other_locations, published_at, source, tags, title, url, work_mode_hint`, none in `seen.db`, all with a description | Python on the server copy, 2026-11-02 | yes |
| `cards` declares `score`, `target`, `work_model` NOT NULL, so changing them needs a table rebuild | `src-tauri/migrations/001_init.sql` | yes |
| `migrate` runs before `foreign_keys = ON`, so a table rebuild inside a migration does not cascade | `src-tauri/src/sqlite_store.rs` `init` | yes |
| A card id is bounded to 64 chars, so a 16-char fingerprint fits | `src-tauri/src/commands.rs` `MAX_ID_LEN`, `001_init.sql` CHECK | yes |
| The frontend reads `score`, `target`, `workModel` in `filter.ts`, `format.ts`, `JobCard.tsx`, `JobDialog.tsx`, `JobCommand.tsx` | `grep` on 2026-11-02 | yes |
| The default filter is `minScore: 70`, which would hide every job without a score unless the rule changes | `src/features/filter-board/model/filter.ts` | yes |
| No frontend code opens a job URL, and no opener plugin is enabled | `src-tauri/capabilities/default.json`, `grep` | yes |
| A debug database with 17 fictional cards exists on the author's machine | `~/.local/share/dev.nandomoreira.jobradar-local/` on 2026-11-02 | yes |
| `tauri-plugin-opener` 2.x exposes a Rust API that needs no webview permission | n/a | assumption (checked when the task compiles) |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN any non-test build opens a database with no cards THEN the system SHALL NOT insert fictional cards, and `sample_cards()` SHALL compile only for tests.
2. **[REQ-002]** WHEN a card has no score, target or work model THEN the store and the IPC SHALL carry `null` for it, existing rows SHALL survive the schema migration unchanged, and the UI SHALL show "Sem nota" and omit the missing target and work model without errors.
3. **[REQ-003]** WHEN filters apply THEN a card without a score SHALL pass any minimum score, and a card without a target SHALL be hidden only when a target filter is active; the Inbox SHALL list cards by score descending, cards without a score after them, then newest first.
4. **[REQ-004]** WHEN the user runs `job-radar-desktop import --seen <file> [--candidates <file>]` THEN the system SHALL insert each `seen.db` row as an unread Inbox card and each `candidates.json` item as a discarded job with the reason "abaixo do corte de score", SHALL print the counts per outcome, SHALL open no window, and SHALL exit 0, or exit 1 with the cause on stderr when a file cannot be read.
5. **[REQ-005]** WHEN import runs again on the same files THEN the system SHALL add nothing, SHALL leave user edits and stage changes intact, SHALL match existing jobs by fingerprint or URL key, SHALL treat `seen.db` as the winner over `candidates.json` for the same job, and SHALL skip every job the user deleted.
6. **[REQ-006]** WHEN import reads a record THEN the system SHALL treat it as untrusted: a URL that is not http(s), a missing title or an invalid fingerprint SHALL skip the record and count it, text SHALL be bounded, the source files SHALL be opened read-only, and one import SHALL commit in a single transaction.
7. **[REQ-007]** WHEN a record is mapped THEN `id` SHALL be the fingerprint, `target` SHALL be derived from the title with the crawler's title families or be null, `work_model` SHALL come from `work_mode_hint` or be null, the card SHALL get one `found` timeline event dated from `sent_at` or `published_at`, and a description SHALL be stored when the record has one.
8. **[REQ-008]** WHEN the user opens the Discarded page from the sidebar THEN the system SHALL list the discarded jobs with title, company, source, location, reason and date, with a readable description, text search and an empty state, operable by keyboard and screen reader.
9. **[REQ-009]** WHEN the user restores a discarded job THEN it SHALL move to the Inbox; WHEN the user deletes one and confirms THEN the system SHALL remove it with its timeline and SHALL keep its fingerprint and URL key so no later import or crawl brings it back.
10. **[REQ-010]** WHEN the user activates a job's link on a card dialog or on the Discarded page THEN the system SHALL open the stored URL in the default browser; the IPC SHALL take the card id, never a URL from the webview, and SHALL refuse a stored value that is not http(s).
11. **[REQ-011]** WHEN the board has no cards THEN the system SHALL show an empty state explaining how to bring jobs in.

### P2 (Next)

1. **[REQ-012]** WHEN this ships THEN README (en, pt-BR) SHALL document the import command, the Discarded page and the link opening in Security, without any server address.

### Out of Scope

- Recalculating scores and fetching job pages (`crawler-core`).
- The `discarded.tsv` rows.
- Importing over the network or from the UI.
- Deleting Inbox cards from the board, and showing the description on the board dialog.
- Removing the 17 fictional cards from an existing debug database automatically; that file is deleted by hand, with the user's consent.

## Edge Cases

- **Same job in both files** - `seen.db` wins; the candidate counts as a duplicate.
- **Two rows with different fingerprints and the same URL key** - the first is kept, the second counts as a duplicate.
- **`sent_at` or `published_at` missing or malformed** - the `found` event uses the import date.
- **`seen.db` without `url_key`** (old copies) - the key is computed by the importer's own URL key function; `crawler-core` proves it matches the crawler.
- **`--candidates` omitted** - only `seen.db` is read.
- **Unknown `work_mode_hint`** - `work_model` is null.
- **`contact_email` invalid** - dropped; a valid one fills `recruiter_email`.
- **Restoring a job already in the Inbox, or deleting an unknown id** - `NotFound`, shown as an alert.
- **Stored URL corrupted to a non-http value** - the open command refuses with `invalid_url`.
- **Importing into a database newer than the app** - the existing refusal applies; exit 1.

## Data Model

```sql
-- migration 2: rebuilds cards so score, target and work_model accept NULL
CREATE TABLE cards_new ( /* same columns and checks as 001, with these changes */
  target       TEXT CHECK (target IN ('ai_llm','full_stack','tech_lead','data')),
  work_model   TEXT CHECK (work_model IN ('remote','hybrid','onsite')),
  score        INTEGER CHECK (score BETWEEN 0 AND 100),
  url_key      TEXT,
  description  TEXT NOT NULL DEFAULT '',
  discard_reason TEXT,
  discarded_at TEXT
);
-- INSERT ... SELECT from cards keeping rowid order; DROP cards; RENAME.
-- Indexes: UNIQUE url_key when not null; discard_reason for the two listings.

CREATE TABLE dismissed_jobs (
  fingerprint TEXT PRIMARY KEY,
  url_key     TEXT,
  dismissed_at TEXT NOT NULL
);
CREATE INDEX dismissed_jobs_url_key ON dismissed_jobs(url_key);
```

`list_cards` returns rows with `discard_reason IS NULL`; the Discarded page returns the others.
`timeline_events.card_id` keeps its foreign key, re-pointed by the rename.

## Architecture Decisions

1. **One `cards` table with `discard_reason`** - A discarded job is a card awaiting a decision; restore clears two columns and keeps its fields, and dedupe has a single place to look.
2. **Tombstones in `dismissed_jobs`** - Delete must beat every later import and crawl, while the row itself goes away.
3. **Import as a CLI subcommand of the same binary** - No UI, no server address and no network; it opens the same database path as the app.
4. **`null` over a sentinel** - A fixed score would rank false data; `Option` keeps the truth through ts-rs to the UI.
5. **Open by id in Rust** - The webview cannot ask the OS to open arbitrary URLs, and no opener permission is granted to it.
6. **Sorting in the board's grouping, not in the store** - The store keeps insertion order for its contract; display order is a UI rule.

## Security

- `seen.db` and `candidates.json` are untrusted input: read-only open, bound parameters, http(s) only, bounded text.
- The URL opened comes from the database by id, re-validated at open time.
- Real jobs stay in the 0600 database; neither source file nor any job text enters the repo. Test fixtures are fictional.
- No new webview permission; CSP unchanged.

## Success Criteria

- `job-radar-desktop import` on the server copies gives 207 Inbox cards and 137 discarded jobs; a second run adds zero.
- All of them stay visible with the default filter, marked "Sem nota".
- Delete, then re-import: the job does not return.
- Rust tests, clippy, fmt and the web gates stay green.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| A 207-card column slows drag and drop | Medium | Measure in the manual check; virtualization stays out of scope unless it is unusable |
| The rebuild migration loses data | High | A test opens a v1 file with edits and asserts every field after migrating |
| Fingerprint rules diverge from the crawler's | High | Import copies the fingerprint from the source; `crawler-core` proves parity with vectors |

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | No fictional cards outside tests | P1 | Pending |
| REQ-002 | Optional score, target, work model through store, IPC and UI | P1 | Pending |
| REQ-003 | Filter rules and Inbox order for jobs without a score | P1 | Pending |
| REQ-004 | Import command | P1 | Pending |
| REQ-005 | Idempotent import, tombstones respected | P1 | Pending |
| REQ-006 | Untrusted input handling | P1 | Pending |
| REQ-007 | Record mapping | P1 | Pending |
| REQ-008 | Discarded page | P1 | Pending |
| REQ-009 | Restore and permanent delete | P1 | Pending |
| REQ-010 | Open job link by id | P1 | Pending |
| REQ-011 | Empty board state | P1 | Pending |
| REQ-012 | README | P2 | Pending |

## Grilling Log

Decisions surfaced and resolved during the 2026-11-02 conversation.

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | Mocks in dev | Removed; tests only | The user wants real jobs on the board |
| 2 | First load | All 207 of `seen.db`; the 137 candidates only on the Discarded page; `discarded.tsv` out | User's choice |
| 3 | Missing fields | Nullable; "Sem nota" until `crawler-core` recalculates | Inventing a score would rank false data |
| 4 | Discarded page actions | Delete for good, plus restore to Inbox | The user asked for delete; curation needs the way back too |
| 5 | Deleted jobs | Tombstone; never return | Otherwise curation undoes itself on each run |
| 6 | File access | Paths given on the command line | The server address cannot enter a public repo |
