---
feature: real-jobs-import
phase: implement
plan_path: docs/plans/2026-11-02-real-jobs-import.md
started_at: 2026-11-02
spec_path: docs/specs/real-jobs-import/SPEC.md
---

# STATE: real-jobs-import

## Current Phase

implement

## Session Log

- 2026-11-02: spec drafted from the conversation decisions; waiting for the user's approval.
- 2026-11-02: spec approved by the user (hard rejects only counted; rescored jobs below the cut stay; opener and crawler.json in scope).
- 2026-11-02: plan saved (11 tasks); validate_plan.py OK. Not implemented.

- 2026-11-02: Tasks 1-11 implemented through migration, import CLI, discarded curation, safe link opening and empty-board UX. Validation and independent review remain pending.
- 2026-11-02: review round 4 by Claude, fixes authorized by the user: plan tests restored verbatim, malformed candidates skipped, URLs trimmed, link failures shown, README updated. Zero findings as self-assessment; independent review, clean-tree proofs and the UI check remain pending.
