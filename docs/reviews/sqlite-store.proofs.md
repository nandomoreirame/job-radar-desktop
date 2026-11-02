# Proofs: sqlite-store

- **Commit:** 5516df5d8e16295ac3fc8aec032cde890d77ea17
- **Tree:** clean
- **Result:** PASS

| Check | Requirements | Proof | Status |
| --- | --- | --- | --- |
| C1 | REQ-001 | run rust \| sqlite_store::tests::moves_and_edits_survive_a_reopen ... ok | PASS |
| C2 | REQ-002 | run rust \| sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok | PASS |
| C3 | REQ-002 | run rust \| store::tests::mock_store_honors_the_store_contract ... ok | PASS |
| C4 | REQ-003 | run rust \| sqlite_store::tests::migrates_once_to_the_latest_version ... ok | PASS |
| C5 | REQ-003 | run rust \| sqlite_store::tests::refuses_a_newer_schema_without_touching_it ... ok | PASS |
| C6 | REQ-004 | run rust \| db_path::tests::picks_the_directory_for_the_build ... ok | PASS |
| C7 | REQ-004 | run rust \| db_path::tests::env_override_wins_unless_empty ... ok | PASS |
| C8 | REQ-004 | run rust \| sqlite_store::tests::database_files_are_private ... ok | PASS |
| C9 | REQ-005 | run rust \| tests::open_store_seeds_only_debug_and_only_when_empty ... ok | PASS |
| C10 | REQ-006 | run rust \| sqlite_store::tests::a_failed_write_leaves_the_data_intact ... ok | PASS |
| C11 | REQ-006 | run rust \| sqlite_store::tests::unknown_stored_values_are_a_storage_error ... ok | PASS |
| C12 | REQ-006 | run rust \| store::tests::storage_errors_hide_the_detail ... ok | PASS |
| C13 | REQ-007 | run rust \| tests::open_store_reports_the_path_and_cause ... ok | PASS |
| C14 | REQ-002 | `(cd src-tauri && cargo test -q) && git diff --exit-code -- src/shared/bindings` | PASS |

## Runs

### run rust (exit 0)

```text
test domain::export_bindings_target ... ok
test sqlite_store::tests::refuses_a_newer_schema_without_touching_it ... ok
test store::tests::an_outcome_outside_closed_is_rejected ... ok
test store::tests::an_unknown_card_is_not_found ... ok
test store::tests::lists_every_seeded_card ... ok
test store::tests::mock_store_honors_the_store_contract ... ok
test sqlite_store::tests::a_failed_write_leaves_the_data_intact ... ok
test store::tests::moving_out_of_closed_clears_the_outcome ... ok
test store::tests::moving_to_closed_requires_an_outcome ... ok
test store::tests::storage_errors_hide_the_detail ... ok
test store::tests::moving_updates_the_stage_for_the_session ... ok
test store::tests::updating_applies_the_details_for_the_session ... ok
test store::tests::updating_an_unknown_card_is_not_found ... ok
test domain::export_bindings_workmodel ... ok
test domain::export_bindings_carddetails ... ok
test domain::export_bindings_timelineevent ... ok
test domain::export_bindings_boardcard ... ok
test tests::open_store_reports_the_path_and_cause ... ok
test sqlite_store::tests::migrates_once_to_the_latest_version ... ok
test sqlite_store::tests::database_files_are_private ... ok
test sqlite_store::tests::seeds_only_an_empty_database ... ok
test sqlite_store::tests::unknown_stored_values_are_a_storage_error ... ok
test sqlite_store::tests::moves_and_edits_survive_a_reopen ... ok
test tests::open_store_seeds_only_debug_and_only_when_empty ... ok
test sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok

test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/main.rs (target/debug/deps/job_radar_desktop-db6316a8c15a255b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests job_radar_desktop_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### `(cd src-tauri && cargo test -q) && git diff --exit-code -- src/shared/bindings` (exit 0)

```text

running 59 tests
...........................................................
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
