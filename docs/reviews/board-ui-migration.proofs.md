# Proofs: board-ui-migration

- **Commit:** 0feb5d33140f2c0968af469b720b5bff6667a443
- **Tree:** clean
- **Result:** PASS

| Check | Requirements | Proof | Status |
| --- | --- | --- | --- |
| C1 | REQ-001 | run web \| ✓ primitives keep radix-ui out of the dependencies | PASS |
| C2 | REQ-001 | run web \| ✓ primitives import no radix primitive under src | PASS |
| C3 | REQ-001 | `pnpm build` | PASS |
| C4 | REQ-002 | run web \| ✓ Board labels each drag handle | PASS |
| C5 | REQ-003 | run web \| ✓ useKanbanColumns asks for an outcome when dropped on Encerradas | PASS |
| C6 | REQ-003 | run web \| ✓ useKanbanColumns restores the store columns | PASS |
| C7 | REQ-003 | run web \| ✓ useKanbanColumns ignores a reorder inside the column | PASS |
| C8 | REQ-004 | run web \| ✓ buildAnnouncements names the column of the card hovered | PASS |
| C9 | REQ-005 | run web \| ✓ JobDialog renders the dialog variant on desktop | PASS |
| C10 | REQ-006 | run web \| ✓ JobCard maps the outcome to a status variant | PASS |
| C11 | REQ-007 | run web \| ✓ JobCommand finds a job without accents | PASS |
| C12 | REQ-007 | run web \| ✓ BoardPage opens a job from the command palette | PASS |
| C13 | REQ-008 | run web \| ✓ Toolbar clamps the minimum score | PASS |
| C14 | REQ-008 | run web \| ✓ Toolbar filters unread only | PASS |
| C15 | REQ-009 | run web \| ✓ AppShell starts collapsed and remembers the choice | PASS |
| C16 | REQ-010 | run web \| ✓ BoardPage has no axe violations with the palette open | PASS |
| C17 | REQ-010 | run web \| ✓ locales have the same keys in pt-BR and en | PASS |

## Runs

### run web (exit 0)

```text
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > names the column of the card hovered 1ms
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > stays quiet when hovering nothing 1ms
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > reports where the card landed 1ms
 ✓ src/widgets/board/model/announcements.test.ts > buildAnnouncements > reports a cancelled drop 1ms
 ✓ src/shared/ui/primitives.test.ts > primitives > keep radix-ui out of the dependencies 2ms
 ✓ src/shared/ui/primitives.test.ts > primitives > import no radix primitive under src 3ms
 ✓ src/pages/board/BoardPage.test.tsx > BoardPage > reports a rejected move 267ms
 ✓ src/pages/board/BoardPage.test.tsx > BoardPage > opens a job from the command palette 277ms
 ✓ src/shared/lib/sidebar-state.test.ts > sidebar state > starts collapsed when nothing was stored 3ms
 ✓ src/shared/lib/sidebar-state.test.ts > sidebar state > round-trips the stored choice 0ms
 ✓ src/shared/lib/sidebar-state.test.ts > sidebar state > survives unavailable storage 2ms
 ✓ src/shared/api/board.test.ts > boardApi > lists the board through the Rust command 3ms
 ✓ src/shared/api/board.test.ts > boardApi > sends the move through the Rust command 0ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > finds the stage holding a card 1ms
 ✓ src/widgets/board/model/kanban.test.ts > columnOf > returns undefined for an unknown id 0ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > moves to another open stage 0ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > asks for an outcome before closing 0ms
 ✓ src/widgets/board/model/kanban.test.ts > resolveKanbanDrop > ignores the same column and a lost card 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts applications in the last seven days 1ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > computes the response rate 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > counts the cards in process 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > averages the days until a response 0ms
 ✓ src/widgets/board/model/metrics.test.ts > computeMetrics > reports no data as null 1ms
 ✓ src/widgets/board/model/group.test.ts > groupByStage > keeps every stage in board order 2ms
 ✓ src/widgets/board/model/group.test.ts > groupByStage > places each card in its stage 0ms
 ✓ src/entities/job/model/format.test.ts > formatThousands > rounds monthly pay to thousands 1ms
 ✓ src/entities/job/model/format.test.ts > scoreTone > splits scores into three tones 0ms
 ✓ src/entities/job/model/format.test.ts > hasNegotiatedSalary > needs at least one regime 0ms
 ✓ src/entities/job/model/format.test.ts > outcomeVariant > maps each outcome to a status variant 0ms

 Test Files  25 passed (25)
      Tests  111 passed (111)
   Start at  04:56:44
   Duration  5.16s (environment 46%, tests 26%, import 15%, setup 10%, transform 2%)

Environment  jsdom was created 25 times · 22.41s total, 46% of tracked time
             create it once per worker with pool: 'vmThreads' (keeps per-file isolation) or isolate: false (shares it across files)
             learn more: https://vitest.dev/guide/improving-performance#test-environments

Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package
```

### `pnpm build` (exit 0)

```text
vite v8.3.5 building client environment for production...
transforming...
✓ 2635 modules transformed.
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
dist/assets/index-DRP1vK_a.css                                  114.55 kB │ gzip:  25.27 kB
dist/assets/index-CxGDlccx.js                                   722.23 kB │ gzip: 230.08 kB

✓ built in 507ms
$ tsc --noEmit && vite build
[plugin builtin:vite-reporter] 
(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rolldownOptions.output.codeSplitting to improve chunking: https://rolldown.rs/reference/OutputOptions.codeSplitting
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
```
