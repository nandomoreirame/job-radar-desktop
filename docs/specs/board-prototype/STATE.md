---
feature: board-prototype
phase: implement
started_at: 2026-11-02
spec_path: docs/specs/board-prototype/SPEC.md
plan_path: docs/plans/2026-11-02-board-prototype.md
---

# STATE: board-prototype

## Current Phase

implement

## Context

First slice of job-radar-desktop: Tauri 2 + React 19 scaffold, Terminal Noir theme with a
glass/flat visual mode validated on WebKitGTK (Niri), Rust domain + `Store` trait with an
in-memory mock served over IPC, and the board screen (six columns, filters, metrics,
drag-and-drop, card dialog) in pt-BR/en.

## Session Log

- 2026-11-02: research done in conversation (stack, storage, Stitch board design); spec approved by the user.
- 2026-11-02: plan saved (14 tasks, 6 batches); validate_plan.py OK after reclassifying 2 replaced test files.
- 2026-11-02: Task 1 done (scaffold, web + Rust suites green).
- 2026-11-02: Task 2 done (Noir tokens, glass/flat mode, shadcn primitives). The WebKitGTK spike (glass vs flat on Niri) was not executed in this automated session; `glass` remains the default and the manual spike is still pending.
- 2026-11-02: Tasks 3-14 done in `--agentic` mode (domain, ts-rs bindings, mock store, IPC,
  seeded cards, filter/metrics models, card and dialog UI, board with drag-and-drop, shell,
  locales, axe gate). One signed commit per task.
- 2026-11-02: Tier 1 green (eslint, tsc, 50 web tests, 29 Rust tests, clippy, build); prettier
  skipped because it is not installed. Tier 2 failed at first on a missing
  `@vitest/coverage-v8`, then passed after adding the provider, 80% thresholds and tests that
  lift coverage to 91/85/87/92.
- 2026-11-02: `check_proofs.py` PASS on a clean tree (C1-C19). Independent review pending.
- 2026-11-02: self-review found two checks that passed without proving their claim: C6 compared
  only the card count and C14 tested accents on company names only. Both tests were
  strengthened in 987984b; `check_proofs.py` PASS again and `validate_verification.py` exits 0.
- 2026-11-02: independent review pending. agy, Codex and Grok all returned exit 3
  (auth/quota/rate-limit) while the network was up, so `docs/reviews/board-prototype.md` is a
  self-assessment with zero open code findings, not a cross-model review. Phase stays
  `implement` and the WebKitGTK glass/flat spike is still a manual step.

## Review Status

- Review file: `docs/reviews/board-prototype.md` (round 1, zero open code findings).
- Independent reviewer: pending. Tried agy, Codex and Grok; all unavailable (exit 3).
- Manual validation pending: `pnpm tauri dev` on Niri for glass vs flat, pointer drag across
  columns and the window controls.
