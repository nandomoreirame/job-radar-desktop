# Proofs: real-jobs-import

- **Commit:** c4638aed76425d105db68dfde8ce80797d176f31
- **Tree:** clean
- **Result:** PASS

| Check | Requirements | Proof | Status |
| --- | --- | --- | --- |
| C1 | REQ-001 | run rust \| tests::open_store_never_seeds ... ok | PASS |
| C2 | REQ-001 | `cd src-tauri && cargo build 2>&1 && ! grep -q 'pub mod mock_data' src/lib.rs` | PASS |
| C3 | REQ-002 | run rust \| sqlite_store::tests::migration_2_keeps_existing_rows ... ok | PASS |
| C4 | REQ-002 | run rust \| sqlite_store::tests::a_card_without_score_round_trips ... ok | PASS |
| C5 | REQ-002 | run web \| ✓ JobCard shows no score label when the score is null | PASS |
| C6 | REQ-003 | run web \| ✓ filterCards keeps a card without a score at any minimum | PASS |
| C7 | REQ-003 | run web \| ✓ groupByStage sorts the inbox by score then newest | PASS |
| C8 | REQ-004 | run rust \| import::tests::routes_seen_to_inbox_and_candidates_to_discarded ... ok | PASS |
| C9 | REQ-004 | run rust \| cli::tests::parses_the_import_command ... ok | PASS |
| C10 | REQ-005 | run rust \| import::tests::importing_twice_changes_nothing ... ok | PASS |
| C11 | REQ-005 | run rust \| import::tests::seen_wins_over_candidates ... ok | PASS |
| C12 | REQ-006 | run rust \| import::tests::skips_and_counts_invalid_records ... ok | PASS |
| C13 | REQ-006 | run rust \| import::tests::unreadable_candidates_change_nothing ... ok | PASS |
| C14 | REQ-006 | run rust \| identity::tests::url_key_matches_the_crawler ... ok | PASS |
| C15 | REQ-007 | run rust \| import::tests::maps_a_record_to_a_card ... ok | PASS |
| C16 | REQ-008 | run web \| ✓ DiscardedPage lists and searches discarded jobs | PASS |
| C17 | REQ-008 | run web \| ✓ DiscardedPage has no axe violations | PASS |
| C18 | REQ-009 | run web \| ✓ DiscardedPage restores a job | PASS |
| C19 | REQ-009 | run web \| ✓ DiscardedPage deletes a job only after confirming | PASS |
| C20 | REQ-009 | run rust \| import::tests::a_deleted_job_does_not_return ... ok | PASS |
| C21 | REQ-010 | run rust \| commands::tests::opens_only_a_stored_http_url ... ok | PASS |
| C22 | REQ-010 | run web \| ✓ JobDialog opens the job link by id | PASS |
| C23 | REQ-011 | run web \| ✓ BoardPage shows the empty state with no cards | PASS |
| C24 | REQ-002 | run rust \| sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok | PASS |

## Runs

### run rust (exit 0)

```text
test import::tests::skips_and_counts_invalid_records ... ok
test store::tests::an_outcome_outside_closed_is_rejected ... ok
test import::tests::seen_wins_over_candidates ... ok
test store::tests::an_unknown_card_is_not_found ... ok
test store::tests::lists_every_seeded_card ... ok
test store::tests::moving_to_closed_requires_an_outcome ... ok
test store::tests::moving_updates_the_stage_for_the_session ... ok
test sqlite_store::tests::migrates_once_to_the_latest_version ... ok
test store::tests::storage_errors_hide_the_detail ... ok
test store::tests::mock_store_honors_the_store_contract ... ok
test store::tests::updating_an_unknown_card_is_not_found ... ok
test store::tests::updating_applies_the_details_for_the_session ... ok
test tests::dialog_needs_a_display ... ok
test store::tests::moving_out_of_closed_clears_the_outcome ... ok
test tests::open_store_reports_the_path_and_cause ... ok
test sqlite_store::tests::database_files_are_private ... ok
test sqlite_store::tests::a_failed_write_leaves_the_data_intact ... ok
test sqlite_store::tests::deleting_keeps_a_tombstone_and_removes_the_timeline ... ok
test sqlite_store::tests::migration_2_keeps_existing_rows ... ok
test tests::open_configured_store_honors_the_env_override ... ok
test sqlite_store::tests::unknown_stored_values_are_a_storage_error ... ok
test sqlite_store::tests::seeds_only_an_empty_database ... ok
test sqlite_store::tests::moves_and_edits_survive_a_reopen ... ok
test tests::open_store_never_seeds ... ok
test sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok

test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running unittests src/main.rs (target/debug/deps/job_radar_desktop-237a8738cd7402ef)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests job_radar_desktop_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### `cd src-tauri && cargo build 2>&1 && ! grep -q 'pub mod mock_data' src/lib.rs` (exit 0)

```text
   Compiling job-radar-desktop v0.1.0 (~/dev/projects/job-radar-desktop/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.93s
```

### run web (exit 0)

```text
 ✓ src/shared/lib/visual-mode.test.ts > readStoredVisualMode > returns the stored choice 1ms
 ✓ src/shared/lib/visual-mode.test.ts > readStoredVisualMode > returns null when storage throws 2ms
 ✓ src/pages/board/BoardPage.test.tsx > BoardPage > saves the notes of a card through the store 1179ms
 ✓ src/pages/board/BoardPage.test.tsx > shows the empty state with no cards 87ms
 ✓ src/shared/ui/primitives.test.ts > primitives > keep radix-ui out of the dependencies 6ms
 ✓ src/shared/ui/primitives.test.ts > primitives > import no radix primitive under src 10ms
 ✓ src/shared/api/board.test.ts > boardApi > lists the board through the Rust command 10ms
 ✓ src/shared/api/board.test.ts > boardApi > sends the move through the Rust command 2ms
 ✓ src/shared/api/board.test.ts > boardApi > sends the details edit through the Rust command 1ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > finds the stage holding a card 5ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > returns undefined for an unknown id 1ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > moves to another open stage 2ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > asks for an outcome before closing 1ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > ignores the same column and a lost card 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts applications in the last seven days 5ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > computes the response rate 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts the cards in process 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > averages the days until a response 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > reports no data as null 2ms
 ✓ src/entities/job/model/details-form.test.ts > toForm > fills every field from the card 2ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > maps empty fields to null and keeps notes as text 1ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > parses the salary in whole reais 0ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > drops a salary that is not a positive number 0ms
 ✓ src/entities/job/model/details-form.test.ts > parseFieldError > reads the field and reason from the store error 0ms
 ✓ src/entities/job/model/details-form.test.ts > parseFieldError > ignores errors that are not about a form field 0ms
 ✓ src/entities/job/model/format.test.ts > formatThousands > rounds monthly pay to thousands 2ms
 ✓ src/entities/job/model/format.test.ts > scoreTone > splits scores into three tones 0ms
 ✓ src/entities/job/model/format.test.ts > hasNegotiatedSalary > needs at least one regime 0ms
 ✓ src/entities/job/model/format.test.ts > outcomeVariant > maps each outcome to a status variant 0ms

 Test Files  27 passed (27)
      Tests  148 passed (148)
   Start at  16:26:03
   Duration  12.60s (environment 46%, tests 29%, import 14%, setup 10%, transform 2%)

Environment  jsdom was created 27 times · 56.70s total, 46% of tracked time
             create it once per worker with pool: 'vmThreads' (keeps per-file isolation) or isolate: false (shares it across files)
             learn more: https://vitest.dev/guide/improving-performance#test-environments

Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package
```
