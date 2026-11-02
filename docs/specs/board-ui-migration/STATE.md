---
feature: board-ui-migration
phase: implement
started_at: 2026-11-02
spec_path: docs/specs/board-ui-migration/SPEC.md
plan_path: docs/plans/2026-11-02-board-ui-migration.md
---

# STATE: board-ui-migration

## Current Phase

implement

## Context

Moves the board-prototype UI from shadcn radix-nova and a custom @dnd-kit board to shadcn
base-nova (Base UI) plus Dice UI: Kanban, ResponsiveDialog, Status, Command palette, Field
filters and a collapsible icon sidebar.

## Session Log

- 2026-11-02: requirements set in conversation (component links, palette, inline filters, collapsed sidebar); spec written.
- 2026-11-02: plan saved (10 tasks, 4 batches); validate_plan.py OK.
- 2026-11-02: Tasks 1-10 implemented sequentially with TDD and one commit per task:
  Base UI and Dice UI primitives, migration locales, Terminal Noir statuses, Kanban state and
  drag handle, ResponsiveDialog, inline filters, command palette, remembered collapsed sidebar,
  and the palette-open axe gate.
- 2026-11-02: Full web suite green (93 tests), coverage green at 93.14/88.05/90.13/96.42 after
  excluding generated shadcn/Dice UI registry files, typecheck, lint, build, Rust 29 tests and
  clippy all pass. Generated Kanban emitted five hook warnings but no lint errors.
- 2026-11-02: Manual Niri validation remains pending: pointer and keyboard drag, close/cancel,
  command palette Enter, Ctrl+B persistence and Terminal Noir status colors.
- 2026-11-02: Independent review was attempted through agy, Codex and Grok; all returned exit 3
  for auth/quota/rate-limit. `docs/reviews/board-ui-migration.md` records a zero-finding
  self-assessment, not independent review. Keep this phase open until an external reviewer and
  the manual Niri checks are available.
- 2026-11-02: round 4 review (Claude panel) left 6 findings. Claude fixed them in 529adab..785c4c8:
  kanban resync on every store update (High), palette twin-jobs test (the Medium finding did not
  reproduce: cmdk 1.1 keeps equal values distinct), columns fit content (local edit in the
  generated KanbanColumn: w-full instead of size-full), scrollbar flush with the window edges,
  inline filter labels, no dialog backdrop blur. tier1, tier2 (coverage), Rust tests and
  check_proofs.py pass (117 web tests).
- 2026-11-02: independent review of the Claude round is pending: agy, Codex and Grok all exited 3,
  and the Claude panel cannot review its own round. The round-4 review file still lists the 6
  findings until an independent reviewer rewrites it. Manual Niri validation still pending.
- 2026-11-02: user-requested round implemented by Claude (13896fa..HEAD): job edit form backed by a
  validated `update_card` Rust command (salary PJ/CLT, process step, next action and date,
  recruiter name/email/LinkedIn, job and application links, notes); "Gerar CV customizado" only in
  the inbox and enabled with a "not implemented yet" notice; "Executar agora" enabled with the same
  notice inside a crawler card; sidebar toggle moved to the title bar (icon only); darker dialog
  overlay; README fixed (`cargo test` must run from `src-tauri/` to regenerate bindings, because
  `--manifest-path` from the root skips `src-tauri/.cargo/config.toml`). tier1, tier2, Rust tests,
  clippy and check_proofs.py (17/17) pass.
- 2026-11-02: independent review still pending: agy, Codex and Grok exited 3 again; Claude cannot
  review its own round. Manual Niri validation pending.
