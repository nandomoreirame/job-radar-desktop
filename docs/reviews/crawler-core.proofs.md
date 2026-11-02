# Proofs: crawler-core

- **Commit:** a7b14e2e35f93d1fc86acdcab59a99739da3690a
- **Tree:** clean
- **Result:** PASS

| Check | Requirements | Proof | Status |
| --- | --- | --- | --- |
| C1 | REQ-001 | run rust \| crawler::identity::tests::matches_the_python_vectors ... ok | PASS |
| C2 | REQ-002 | run rust \| crawler::filters::tests::matches_the_python_vectors ... ok | PASS |
| C3 | REQ-002 | run rust \| crawler::scoring::tests::matches_the_python_vectors ... ok | PASS |
| C4 | REQ-002 | run rust \| crawler::profile::tests::matches_the_python_skills ... ok | PASS |
| C5 | REQ-003 | run rust \| crawler::config::tests::refuses_a_missing_or_empty_profile ... ok | PASS |
| C6 | REQ-003 | run rust \| crawler::config::tests::reads_defaults_and_names_the_invalid_field ... ok | PASS |
| C7 | REQ-004 | run rust \| crawler::http::tests::honors_robots_pacing_and_retries ... ok | PASS |
| C8 | REQ-004 | run rust \| crawler::http::tests::bounds_the_body_and_refuses_other_schemes ... ok | PASS |
| C9 | REQ-004 | run rust \| crawler::robots::tests::matches_the_python_vectors ... ok | PASS |
| C10 | REQ-005 | run rust \| crawler::sources::gupy::tests::parses_the_recorded_search_like_python ... ok | PASS |
| C11 | REQ-005 | run rust \| crawler::sources::solides::tests::parses_the_recorded_pages_like_python ... ok | PASS |
| C12 | REQ-006 | run rust \| crawler::pipeline::tests::routes_new_jobs_and_skips_known_ones ... ok | PASS |
| C13 | REQ-006 | run rust \| crawler::pipeline::tests::a_failing_source_does_not_stop_the_others ... ok | PASS |
| C14 | REQ-007 | run rust \| crawler::pipeline::tests::rescoring_keeps_user_edits_and_stage ... ok | PASS |
| C15 | REQ-007 | run rust \| crawler::pipeline::tests::a_stored_description_needs_no_request ... ok | PASS |
| C16 | REQ-007 | run rust \| crawler::pipeline::tests::a_low_rescore_stays_in_the_inbox ... ok | PASS |
| C17 | REQ-008 | run rust \| crawler::pipeline::tests::closed_discards_and_failures_keep ... ok | PASS |
| C18 | REQ-008 | run rust \| crawler::pipeline::tests::circuit_breaker_stops_a_mass_discard ... ok | PASS |
| C19 | REQ-008 | run rust \| crawler::sources::tests::detail_outcomes_follow_the_closed_rules ... ok | PASS |
| C20 | REQ-009 | run rust \| crawler::runner::tests::refuses_a_second_run ... ok | PASS |
| C21 | REQ-009 | run web \| ✓ CrawlerCard disables the button while running | PASS |
| C22 | REQ-009 | run rust \| sqlite_store::tests::the_last_run_survives_a_reopen ... ok | PASS |
| C23 | REQ-010 | run web \| ✓ CrawlerCard explains a never run and a refused run | PASS |

## Runs

### run rust (exit 0)

```text
test store::tests::an_outcome_outside_closed_is_rejected ... ok
test store::tests::an_unknown_card_is_not_found ... ok
test store::tests::lists_every_seeded_card ... ok
test store::tests::mock_store_honors_the_store_contract ... ok
test store::tests::moving_out_of_closed_clears_the_outcome ... ok
test store::tests::moving_to_closed_requires_an_outcome ... ok
test store::tests::moving_updates_the_stage_for_the_session ... ok
test sqlite_store::tests::a_failed_write_leaves_the_data_intact ... ok
test store::tests::storage_errors_hide_the_detail ... ok
test store::tests::updating_an_unknown_card_is_not_found ... ok
test store::tests::updating_applies_the_details_for_the_session ... ok
test tests::dialog_needs_a_display ... ok
test sqlite_store::tests::lists_only_cards_needing_a_score ... ok
test tests::open_store_reports_the_path_and_cause ... ok
test sqlite_store::tests::migrates_once_to_the_latest_version ... ok
test crawler::sources::gupy::tests::collect_searches_every_term_in_the_path_and_keeps_the_good_ones ... ok
test crawler::sources::gupy::tests::parses_the_recorded_search_like_python ... ok
test sqlite_store::tests::the_last_run_survives_a_reopen ... ok
test sqlite_store::tests::migration_2_keeps_existing_rows ... ok
test tests::open_configured_store_honors_the_env_override ... ok
test sqlite_store::tests::seeds_only_an_empty_database ... ok
test sqlite_store::tests::unknown_stored_values_are_a_storage_error ... ok
test sqlite_store::tests::moves_and_edits_survive_a_reopen ... ok
test tests::open_store_never_seeds ... ok
test sqlite_store::tests::sqlite_store_honors_the_store_contract ... ok

test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running unittests src/main.rs (target/debug/deps/job_radar_desktop-05711e2d495a130c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests job_radar_desktop_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### run web (exit 0)

```text
 ✓ src/shared/lib/sidebar-state.test.ts > sidebar state > survives unavailable storage 5ms
 ✓ src/entities/job/model/details-form.test.ts > toForm > fills every field from the card 6ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > maps empty fields to null and keeps notes as text 3ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > parses the salary in whole reais 1ms
 ✓ src/entities/job/model/details-form.test.ts > toDetails > drops a salary that is not a positive number 1ms
 ✓ src/entities/job/model/details-form.test.ts > parseFieldError > reads the field and reason from the store error 1ms
 ✓ src/entities/job/model/details-form.test.ts > parseFieldError > ignores errors that are not about a form field 1ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > finds the stage holding a card 3ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > returns undefined for an unknown id 0ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > moves to another open stage 1ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > asks for an outcome before closing 0ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > ignores the same column and a lost card 1ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > keeps every card with empty filters 3ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > filters by source and target 0ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > drops cards below the minimum score 0ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > keeps only unread cards when asked 0ms
 ✓ src/features/filter-board/model/filter.test.ts > matchesText > matches titles and companies without accents 1ms
 ✓ src/features/filter-board/model/filter.test.ts > clampScore > keeps the score between 0 and 100 0ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > keeps a card without a score at any minimum 0ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > hides a card without a target only when a target is selected 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts applications in the last seven days 2ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > computes the response rate 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts the cards in process 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > averages the days until a response 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > reports no data as null 1ms
 ✓ src/entities/job/model/format.test.ts > formatThousands > rounds monthly pay to thousands 2ms
 ✓ src/entities/job/model/format.test.ts > scoreTone > splits scores into three tones 0ms
 ✓ src/entities/job/model/format.test.ts > hasNegotiatedSalary > needs at least one regime 0ms
 ✓ src/entities/job/model/format.test.ts > outcomeVariant > maps each outcome to a status variant 0ms

 Test Files  30 passed (30)
      Tests  170 passed (170)
   Start at  19:14:39
   Duration  14.18s (environment 45%, tests 29%, import 14%, setup 10%, transform 2%)

Environment  jsdom was created 30 times · 62.94s total, 45% of tracked time
             create it once per worker with pool: 'vmThreads' (keeps per-file isolation) or isolate: false (shares it across files)
             learn more: https://vitest.dev/guide/improving-performance#test-environments

Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package
```
