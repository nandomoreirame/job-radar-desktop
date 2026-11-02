# Spec: Board UI Migration (shadcn Base UI + Dice UI)

## Problem Statement

The board prototype ships hand-made UI on shadcn's Radix style plus a custom @dnd-kit board.
The author wants the app built on shadcn's Base UI components and Dice UI, so the board, the
job dialog, status badges, search, filters and sidebar come from maintained primitives with
keyboard support instead of local code.

## Goals

- Every UI primitive comes from shadcn `base-nova` (Base UI) or Dice UI; `radix-ui` leaves the
  direct dependency tree and is not imported under `src` (cmdk may retain a transitive dialog
  dependency).
- The board runs on Dice UI Kanban, with pointer and keyboard drag (Space/Enter, arrows, Esc).
- Search moves to a Ctrl/Cmd+K command palette that opens the job dialog.
- The sidebar collapses to icons by default and remembers the user's choice.
- All existing behavior (move rules, filters, metrics, i18n, axe gate) keeps passing.

## Context

`board-prototype` is implemented (commits `9c80bdc..9850a30`): Tauri 2 + React 19, shadcn
`radix-nova` primitives (button, badge, dialog, dropdown-menu, input), a custom `@dnd-kit/core`
board with `resolveDrop` and `buildAnnouncements`, a toolbar with search, dropdown filters and a
range input, and a hand-made sidebar. Its glass/flat WebKitGTK spike and independent review are
still pending; this slice does not change either.

Component choices were made by the user on 2026-11-02, with these links: Dice UI
`base/kanban`, `base/responsive-dialog`, `base/status`; shadcn `base/command`, `base/field`
(plus input, select, checkbox), `base/sidebar`.

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| Dice UI kanban depends on `@base-ui/react`, `@base-ui/utils`, `@dnd-kit/core`, `@dnd-kit/modifiers`, `@dnd-kit/sortable`, `@dnd-kit/utilities` | `https://diceui.com/r/kanban.json` (`dependencies`), fetched 2026-11-02 | yes |
| Kanban moves an item across columns during `onDragOver` through `onValueChange`, calls the consumer's `onDragEnd`/`onDragCancel` first, and does not restore columns on cancel | `kanban.tsx` `onDragOver`/`onDragEnd`/`onDragCancel` in the same registry item | yes |
| Kanban merges a consumer `accessibility` prop over its English announcements | `kanban.tsx` `accessibility={{ announcements, screenReaderInstructions, ...accessibility }}` | yes |
| `KanbanItemHandle` renders a `<button>` carrying the sortable attributes and listeners | `kanban.tsx` `KanbanItemHandle` | yes |
| ResponsiveDialog switches to a vaul-free Base UI drawer below `breakpoint = 768` via `useIsMobile` (matchMedia) | `https://diceui.com/r/responsive-dialog.json` | yes |
| Status exposes `default`, `success`, `error`, `warning`, `info` variants and sets `data-variant` | `https://diceui.com/r/status.json` | yes |
| shadcn 4.23 supports `--base base` and the `base-nova` preset; `https://ui.shadcn.com/r/styles/base-nova/sidebar.json` exists | `shadcn init --help`, HTTP 200 on 2026-11-02 | yes |
| base-nova `CommandDialog` renders `children` inside `DialogContent` without a `Command` wrapper and hides title/description visually | `https://ui.shadcn.com/r/styles/base-nova/command.json` | yes |
| base-nova `SidebarProvider` toggles on Ctrl/Cmd+B, accepts `open`/`onOpenChange`, and persists only through a cookie | `https://ui.shadcn.com/r/styles/base-nova/sidebar.json` | yes |
| Radix-only code today: `asChild` in `JobDialog`, `Toolbar`, `MoveMenu`; `onSelect` on menu items | `src/entities/job/ui/JobDialog.tsx:82`, `src/features/filter-board/ui/Toolbar.tsx:38`, `src/features/move-card/ui/MoveMenu.tsx:24` | yes |
| jsdom has no `matchMedia`, which `useIsMobile` calls | n/a | assumption (polyfilled in test setup) |
| Base UI `Select` supports `multiple` with a function child in `SelectValue` | n/a | assumption (Base UI 1.9 docs; verify in Task 7) |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN the UI layer is built THEN the system SHALL take its primitives from shadcn `base-nova` (Base UI) and Dice UI, and `radix-ui` SHALL not be a dependency nor be imported under `src`.
2. **[REQ-002]** WHEN the board renders THEN the system SHALL use Dice UI Kanban with the six stage columns fixed in order, each card draggable through a labeled handle by pointer and by keyboard (Space/Enter to pick up and drop, arrow keys to move, Esc to cancel).
3. **[REQ-003]** WHEN a drag ends in another column THEN the system SHALL call `move_card` for that stage, SHALL open the outcome dialog when the column is Encerradas, and SHALL return the card to its original column when the drag or the outcome dialog is cancelled; a reorder inside the same column SHALL not call `move_card`.
4. **[REQ-004]** WHEN a card is dragged THEN the system SHALL narrate pick-up, move, drop and cancel, and the drag instructions, from the active locale.
5. **[REQ-005]** WHEN the user activates a card THEN the system SHALL open the job details in a Dice UI ResponsiveDialog with the same content as before (details, salary, artifacts, timeline, disabled "Gerar CV customizado"), closable with Esc.
6. **[REQ-006]** WHEN a card shows an outcome, an interview step or a running AI CLI THEN the system SHALL render a Dice UI Status: Contratado as success, Recusada as error, Descartada and Sem resposta as default, the interview step as warning, and the running CLI as info with a live indicator, all colored with the Terminal Noir tokens.
7. **[REQ-007]** WHEN the user presses Ctrl/Cmd+K or the search button THEN the system SHALL open a shadcn Command palette that lists the jobs by title and company with accent-insensitive matching, and SHALL open the job dialog for the selected job.
8. **[REQ-008]** WHEN the toolbar renders THEN the system SHALL show inline shadcn Field controls: multi-select Fonte and Alvo, a numeric Score mín. clamped to 0-100, and a "Somente não lidas" checkbox, and the board and metrics SHALL follow them.
9. **[REQ-009]** WHEN the app opens THEN the system SHALL render the shadcn Sidebar collapsed to icons by default, toggle it with Ctrl/Cmd+B or its trigger, remember the last state across launches, show item labels as tooltips while collapsed, and keep `aria-current` on the active page.
10. **[REQ-010]** WHEN the migration is complete THEN the board page SHALL still have no axe-core WCAG 2.1 A/AA violations, including with the palette open, and the pt-BR and en locales SHALL keep identical key sets.

## Edge Cases

- **Drag cancelled after crossing columns** - Kanban already moved the card in local state during `onDragOver`; the board restores the snapshot taken at drag start.
- **Drop on Encerradas, then Cancelar** - the card returns to its origin column; nothing reaches `move_card`.
- **`move_card` rejects the move** - the board resyncs from the store's cards, so the card snaps back and the existing alert shows.
- **Reorder inside a column** - visual only for the session; the domain has no order field, so the next store refresh restores the store order.
- **Score mín. typed out of range or empty** - clamped to 0-100; empty or non-numeric becomes 0.
- **Palette with no match** - shows "Nenhuma vaga encontrada."; Esc closes without opening anything.
- **Duplicate titles in the palette** - each item value includes the card id, so cmdk keeps them distinct.
- **Ctrl+K while the palette is open** - keeps it open (idempotent).
- **Local storage unavailable** - the sidebar falls back to collapsed and never throws.
- **Window narrower than 768 px** - not reachable (window min width 1280); ResponsiveDialog would render the drawer variant.

## Architecture Decisions

1. **Whole UI layer on Base UI** - Chose shadcn `base-nova` for every primitive over mixing Radix and Base UI, because the Dice UI components the user picked are the Base UI variants and two primitive libraries would double the bundle and the styling rules. Alternative discarded: Dice UI Radix variants (not what the user linked).
2. **Dice UI Kanban replaces the custom @dnd-kit board** - Keyboard drag, sortable columns and overlay come with it; `resolveDrop` becomes a stage comparison between the drag origin and the column holding the card at drop time.
3. **Board keeps a local column snapshot** - Kanban mutates columns during the drag; a `useKanbanColumns` hook owns that local state, the origin of the active card, and the restore on cancel. The store stays the source of truth and resyncs the columns when cards change.
4. **Labeled drag handle, not the whole card** - `KanbanItem asHandle` would make the card a button containing other buttons (axe `nested-interactive`); a grip `KanbanItemHandle` beside the move menu keeps the title button and the menu valid.
5. **Search is a palette, filters stay inline** - Free-text query leaves `BoardFilters`; the palette jumps to a job instead of filtering columns (user decision).
6. **Sidebar state in localStorage** - Controlled `open` on `SidebarProvider` with a small storage helper, because the built-in cookie persistence targets server rendering.
7. **Translated drag narration** - Dice UI's English announcements are overridden through the `accessibility` prop with i18n strings.

## Grilling Log

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | Who implements | Spec + plan here; opencode implements | User choice, same flow as board-prototype |
| 2 | Command palette role | Ctrl/Cmd+K palette that opens the job dialog | User choice |
| 3 | Filters placement | Inline toolbar with shadcn Field controls | User choice |
| 4 | Sidebar initial state | Collapsed to icons, Ctrl/Cmd+B expands, state remembered | User choice |
| 5 | Radix and Base UI side by side | No; the whole layer moves to base-nova | Dice UI links are Base UI variants |
| 6 | Free-text filter | Removed from the toolbar; the palette replaces it | Follows decision 2 |
| 7 | Checkbox filter | "Somente não lidas" | User asked for checkboxes in the filter form; unread is the one boolean the cards carry |
| 8 | Move menu | Kept on each card | Only path to "Encerrar como" with an explicit outcome |

## Constraints

- Linux/WebKitGTK primary target; window min width 1280, so the drawer variant never shows in practice.
- No change to the Rust side, IPC contract or ts-rs bindings.
- Terminal Noir palette must override the stock colors of Status and Sidebar.
- Repository is public: no personal paths or hosts in code.

## Out of Scope

| Item | Reason |
|------|--------|
| Persisting card order inside a column | Domain has no order field; needs a store change |
| Draggable columns | Stage order is fixed by the domain |
| Glass/flat WebKitGTK spike and independent review of board-prototype | Tracked in board-prototype STATE |
| New pages (profile, resumes, prep, settings) | Later plans |

## Test Strategy

- **Unit:** pure helpers (`columnOf`, `resolveKanbanDrop`, `clampScore`, `outcomeVariant`, sidebar storage, filter model) and the `useKanbanColumns` hook with `renderHook`.
- **Component:** Testing Library over Status in cards, the filter toolbar (Select, Checkbox, Input), the palette, the responsive job dialog, the move menu and the sidebar; drag itself is covered through the hook because dnd-kit cannot measure layout in jsdom.
- **Accessibility:** axe-core over the board page, with and without the palette open.
- **Manual:** `pnpm tauri dev` on Niri: pointer and keyboard drag, drop on Encerradas with cancel, Ctrl+K, Ctrl+B, sidebar state after restart.

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | Primitives from base-nova and Dice UI, no radix-ui | P1 | Pending |
| REQ-002 | Dice UI Kanban board with keyboard drag handle | P1 | Pending |
| REQ-003 | Drop rules, outcome dialog, restore on cancel | P1 | Pending |
| REQ-004 | Translated drag narration | P1 | Pending |
| REQ-005 | Job details in ResponsiveDialog | P1 | Pending |
| REQ-006 | Dice UI Status for outcome, step and AI CLI | P1 | Pending |
| REQ-007 | Ctrl/Cmd+K command palette | P1 | Pending |
| REQ-008 | Inline Field filters with clamped score | P1 | Pending |
| REQ-009 | Collapsible icon sidebar with remembered state | P1 | Pending |
| REQ-010 | axe gate and locale parity preserved | P1 | Pending |
