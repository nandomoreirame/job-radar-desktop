# Lessons

<!-- Rendered by ~/.claude/scripts/lessons.py; never edit by hand. -->

## Confirmed

None.

## Candidates

- **L-001** `startup` (ac_gap): Verify startup failure paths by running the real binary; a unit test of the helper does not prove the dialog or the exit status.
  - Features: sqlite-store; sources: C13
- **L-002** `tests` (weak_test): A check must exercise the function production calls; when a fix adds a new entry point, replace the old one instead of leaving it covered by the tests.
  - Features: sqlite-store; sources: C6
- **L-003** `tests` (weak_test): A test that claims a refused operation leaves a file untouched must compare the file bytes, not derived fields.
  - Features: sqlite-store; sources: C5
- **L-004** `process` (review_finding): An implementer closing a review round with itself as reviewer does not close the round; record the review as pending.
  - Features: sqlite-store; sources: docs/reviews/sqlite-store.rounds.jsonl
- **L-005** `crawler` (review_finding): A parser of a third-party page treats a missing expected collection as an error, and only an existing empty collection as zero results.
  - Features: crawler-core; sources: src-tauri/src/crawler/sources/gupy.rs parse_search
- **L-006** `crawler` (review_finding): A request built from a stored or imported URL is checked against the hosts the docs declare before it is made.
  - Features: crawler-core; sources: src-tauri/src/crawler/pipeline.rs rescore

## Quarantined

None.
