# Proofs: board-prototype

- **Commit:** 987984b398dce390cfba3ed2b41031060841a28d
- **Tree:** clean
- **Result:** PASS

| Check | Requirements | Proof | Status |
| --- | --- | --- | --- |
| C1 | REQ-001 | `pnpm build` | PASS |
| C2 | REQ-001 | `pnpm lint` | PASS |
| C3 | REQ-002 | run web \| ✓ resolveVisualMode defaults to glass | PASS |
| C4 | REQ-003 | run rust \| domain::tests::stage_serializes_as_snake_case ok | PASS |
| C5 | REQ-003 | `[ -f src/shared/bindings/BoardCard.ts ]` | PASS |
| C6 | REQ-004 | run rust \| commands::tests::list_board_returns_the_seeded_cards ok | PASS |
| C7 | REQ-005 | run rust \| store::tests::moving_to_closed_requires_an_outcome ok | PASS |
| C8 | REQ-005 | run rust \| commands::tests::move_card_rejects_an_oversized_id ok | PASS |
| C9 | REQ-006 | run web \| ✓ App shows a placeholder for pages not built yet | PASS |
| C10 | REQ-006 | run web \| ✓ AppShell drives the window controls | PASS |
| C11 | REQ-007 | run web \| ✓ Board renders the six columns in board order | PASS |
| C12 | REQ-008 | run web \| ✓ JobCard shows only PJ when CLT was not negotiated | PASS |
| C13 | REQ-009 | run web \| ✓ JobCard shows the outcome of a closed card | PASS |
| C14 | REQ-010 | run web \| ✓ filterCards matches company names without accents | PASS |
| C15 | REQ-010 | run web \| ✓ computeMetrics computes the response rate | PASS |
| C16 | REQ-011 | run web \| ✓ Board closes a card with the chosen outcome | PASS |
| C17 | REQ-012 | run web \| ✓ JobDialog keeps CV generation disabled until it exists | PASS |
| C18 | REQ-013 | run web \| ✓ locales have the same keys in pt-BR and en | PASS |
| C19 | REQ-014 | run web \| ✓ BoardPage has no axe violations | PASS |

## Runs

### `pnpm build` (exit 0)

```text
vite v8.3.5 building client environment for production...
transforming...
✓ 2219 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                                                   0.39 kB │ gzip:   0.26 kB
dist/assets/jetbrains-mono-greek-400-normal-C190GLew.woff2        4.22 kB
dist/assets/jetbrains-mono-cyrillic-400-normal-BEIGL1Tu.woff2     5.32 kB
dist/assets/jetbrains-mono-vietnamese-400-normal-CqNFfHCs.woff    5.37 kB
dist/assets/jetbrains-mono-greek-400-normal-B9oWc5Lo.woff         5.66 kB
dist/assets/jetbrains-mono-cyrillic-400-normal-ugxPyKxw.woff      6.97 kB
dist/assets/jetbrains-mono-latin-ext-400-normal-Bc8Ftmh3.woff2    7.33 kB
dist/assets/jetbrains-mono-latin-ext-400-normal-fXTG6kC5.woff    10.12 kB
dist/assets/inter-vietnamese-wght-normal-CBcvBZtf.woff2          10.25 kB
dist/assets/inter-greek-ext-wght-normal-DlzME5K_.woff2           11.23 kB
dist/assets/inter-cyrillic-wght-normal-DqGufNeO.woff2            18.74 kB
dist/assets/inter-greek-wght-normal-CkhJZR-_.woff2               18.99 kB
dist/assets/jetbrains-mono-latin-400-normal-V6pRDFza.woff2       21.16 kB
dist/assets/inter-cyrillic-ext-wght-normal-BOeWTOD4.woff2        25.96 kB
dist/assets/jetbrains-mono-latin-400-normal-6-qcROiO.woff        27.49 kB
dist/assets/inter-latin-wght-normal-Dx4kXJAl.woff2               48.25 kB
dist/assets/inter-latin-ext-wght-normal-DO1Apj_S.woff2           85.06 kB
dist/assets/noir-skyline-DGz1OufM.png                           208.94 kB
dist/assets/index-D--UmjR3.css                                   55.23 kB │ gzip:  16.46 kB
dist/assets/index-377Mpm7W.js                                   491.56 kB │ gzip: 155.01 kB

✓ built in 268ms
$ tsc --noEmit && vite build
```

### `pnpm lint` (exit 0)

```text
$ eslint .
```

### run web (exit 0)

```text
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > reports where the card landed 2ms
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > reports a cancelled drop 2ms
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > has no column name for unknown targets 1ms
 ✓ src/shared/api/board.test.ts > boardApi > lists the board through the Rust command 5ms
 ✓ src/shared/api/board.test.ts > boardApi > sends the move through the Rust command 1ms
 ✓ src/widgets/board/model/drop.test.ts > resolveDrop > moves to another open stage 6ms
 ✓ src/widgets/board/model/drop.test.ts > resolveDrop > asks for an outcome before closing 1ms
 ✓ src/widgets/board/model/drop.test.ts > resolveDrop > ignores a drop on the card's own column 1ms
 ✓ src/widgets/board/model/drop.test.ts > resolveDrop > ignores unknown columns and missing data 0ms
 ✓ src/entities/job/model/format.test.ts > formatThousands > rounds monthly pay to thousands 3ms
 ✓ src/entities/job/model/format.test.ts > scoreTone > splits scores into three tones 1ms
 ✓ src/entities/job/model/format.test.ts > hasNegotiatedSalary > needs at least one regime 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts applications in the last seven days 5ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > computes the response rate 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts the cards in process 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > averages the days until a response 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > reports no data as null 2ms
 ✓ src/shared/lib/visual-mode.test.ts > resolveVisualMode > prefers the stored mode 6ms
 ✓ src/shared/lib/visual-mode.test.ts > resolveVisualMode > falls back to the env default 1ms
 ✓ src/shared/lib/visual-mode.test.ts > resolveVisualMode > defaults to glass 1ms
 ✓ src/shared/lib/visual-mode.test.ts > applyVisualMode > sets data-visual on the root 4ms
 ✓ src/shared/lib/visual-mode.test.ts > readStoredVisualMode > returns the stored choice 1ms
 ✓ src/shared/lib/visual-mode.test.ts > readStoredVisualMode > returns null when storage throws 3ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > keeps every card with empty filters 6ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > matches company names without accents 2ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > filters by source and target 1ms
 ✓ src/features/filter-board/model/filter.test.ts > filterCards > drops cards below the minimum score 1ms
 ✓ src/widgets/board/model/group.test.ts > groupByStage > keeps every stage in board order 2ms
 ✓ src/widgets/board/model/group.test.ts > groupByStage > places each card in its stage 0ms

 Test Files  20 passed (20)
      Tests  78 passed (78)
   Start at  04:02:08
   Duration  6.15s (environment 47%, tests 30%, import 12%, setup 9%, transform 2%)

Environment  jsdom was created 20 times · 26.53s total, 47% of tracked time
             create it once per worker with pool: 'vmThreads' (keeps per-file isolation) or isolate: false (shares it across files)
             learn more: https://vitest.dev/guide/improving-performance#test-environments

Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package
```

### run rust (exit 0)

```text
test domain::tests::outcome_serializes_as_snake_case ... ok
test domain::tests::stage_order_matches_the_board ... ok
test domain::tests::salary_keeps_a_missing_regime_as_null ... ok
test domain::tests::stage_serializes_as_snake_case ... ok
test mock_data::tests::cards_before_applying_have_no_salary ... ok
test mock_data::tests::closed_cards_carry_an_outcome ... ok
test mock_data::tests::seeds_every_board_stage ... ok
test mock_data::tests::ids_are_unique ... ok
test store::tests::an_unknown_card_is_not_found ... ok
test store::tests::an_outcome_outside_closed_is_rejected ... ok
test store::tests::moving_out_of_closed_clears_the_outcome ... ok
test store::tests::moving_to_closed_requires_an_outcome ... ok
test store::tests::lists_every_seeded_card ... ok
test store::tests::moving_updates_the_stage_for_the_session ... ok
test domain::export_bindings_artifacts ... ok
test domain::export_bindings_eventkind ... ok
test domain::export_bindings_target ... ok
test domain::export_bindings_salary ... ok
test domain::export_bindings_stage ... ok
test domain::export_bindings_outcome ... ok
test domain::export_bindings_workmodel ... ok
test domain::export_bindings_timelineevent ... ok
test domain::export_bindings_boardcard ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (src-tauri/target/debug/deps/job_radar_desktop_lib-d6dda9757b55956e)
     Running unittests src/main.rs (src-tauri/target/debug/deps/job_radar_desktop-0c341fe8e4ec7d4d)
   Doc-tests job_radar_desktop_lib
```

### `[ -f src/shared/bindings/BoardCard.ts ]` (exit 0)

```text

```
