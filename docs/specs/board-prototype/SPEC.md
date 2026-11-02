# Spec: Board Prototype (setup + board screen with mocks)

## Problem Statement

job-radar finds and scores job postings, but it stops at a Discord message: nothing tracks what
happened after an application (reply, interview stages, offer, rejection, silence), so the search
cannot learn from its own results. `job-radar-desktop` is the open-source, local-first desktop app
that will own that tracking. This first slice proves the stack and the main screen before any real
data or AI integration exists.

## Goals

- `pnpm tauri dev` opens a working Tauri 2 + React 19 app on Linux (Niri/Wayland, WebKitGTK).
- The Terminal Noir board renders six columns from Rust-served mock data, readable on a
  2560x1080 ultrawide without horizontal scroll.
- Cards move between columns by drag-and-drop and by a keyboard-accessible menu, and open in a
  dialog.
- The glass visual (backdrop blur) is validated on WebKitGTK, with a flat fallback ready.

## Context

New repository; nothing exists yet. Decisions taken in the design conversation (2026-11-02):

- Product: open source, local-first. One binary will later serve three modes: GUI, `crawl`
  (headless, for a timer) and `mcp` (stdio). No HTTP server and no open port: the frontend talks
  to Rust through Tauri IPC.
- Storage: SQLite by default; Postgres on a self-hosted machine as an optional mode, both behind a
  `Store` trait. This slice ships only the trait and an in-memory mock.
- The existing Python crawler (`job-radar`, separate repo) keeps running until a Rust port reaches
  parity; it is not touched here.
- Visual reference: Stitch project `4820420849267082439`, screen "Job Radar — Board Ultrawide
  2560x1080" (`e37b075538264e7f9f317e1e39ff16fb`). Background art: the Terminal Noir wallpaper
  (`~/.dotfiles/wallpapers/terminal-noir-noir.png`, the author's own artwork).

## Verified Claims

| Claim | Source (file:line) | Verified |
|-------|--------------------|----------|
| `pnpm create tauri-app <dir> -m pnpm -t react-ts -y` scaffolds Tauri 2 + React + Vite 8 + TS 6 non-interactively | trial run in scratchpad, 2026-11-02 | yes |
| Toolchain present: rustc 1.96, pnpm 11.5, node 24.16, webkit2gtk-4.1 2.54.2 | local `--version` / `pacman -Q`, 2026-11-02 | yes |
| Latest stable crates: tauri 2.12.5, tauri-build 2.7.4, ts-rs 12.0.1 | `cargo info`, 2026-11-02 | yes |
| Wallpaper exists and is 208 KB | `~/.dotfiles/wallpapers/terminal-noir-noir.png` | yes |
| WebKitGTK renders `backdrop-filter` smoothly on Niri | n/a | assumption (validated by Task 2 spike) |

## Requirements

### P1 (MVP)

1. **[REQ-001]** WHEN the developer runs the project scripts THEN the system SHALL build, type-check, lint and pass the web and Rust test suites, and `pnpm tauri dev` SHALL open a frameless window titled job-radar.
2. **[REQ-002]** WHEN the app starts THEN the system SHALL render the Terminal Noir skyline behind the board with glass panels by default, and SHALL use flat panels (solid fill, borders, no backdrop blur) when the visual mode resolves to flat from local storage or `VITE_VISUAL_MODE`.
3. **[REQ-003]** WHEN a board card crosses the IPC boundary THEN the system SHALL serialize it with camelCase fields and snake_case enum values, and SHALL keep the TypeScript types generated from the Rust domain by ts-rs.
4. **[REQ-004]** WHEN the frontend requests the board THEN the system SHALL return the seeded mock cards from the Rust `MockStore` through the `list_board` command, behind the `Store` trait.
5. **[REQ-005]** WHEN a card is moved through `move_card` THEN the system SHALL persist the new stage for the session, SHALL require an outcome when the target is closed, and SHALL reject an outcome for any other stage, an empty or oversized id, or an unknown id.
6. **[REQ-006]** WHEN the app opens THEN the system SHALL show a custom title bar with minimize, maximize and close, and a sidebar with Board, Meu perfil, Meus currículos, Preparação and Configuração, where every item except Board shows a placeholder page.
7. **[REQ-007]** WHEN the board loads THEN the system SHALL render the columns Caixa de entrada, Triagem, Candidatado, Em processo, Oferta and Encerradas in that order with their card counts, each card showing title, company, source, target, work model, score, artifact state, last activity or next action, and an AI-running indicator when present.
8. **[REQ-008]** WHEN a card has a negotiated salary THEN the system SHALL show the PJ and/or CLT monthly values with their labels, and SHALL show no salary line when neither was negotiated.
9. **[REQ-009]** WHEN a card is in Encerradas THEN the system SHALL show its outcome badge: Contratado, Recusada, Descartada or Sem resposta.
10. **[REQ-010]** WHEN the user types in the search or changes the source, target or minimum-score filters THEN the system SHALL show only matching cards (accent-insensitive match on title and company) and SHALL compute the metrics strip from the visible cards.
11. **[REQ-011]** WHEN the user drags a card to another column or picks a destination in the card's "Mover para" menu THEN the system SHALL move the card through `move_card`, asking for the outcome when the destination is Encerradas.
12. **[REQ-012]** WHEN the user activates a card THEN the system SHALL open an accessible dialog with the job details, negotiated salary, artifacts and timeline, with a disabled "Gerar CV customizado" button marked "Em breve", closable with Esc.
13. **[REQ-013]** WHEN the UI renders text THEN the system SHALL take it from the pt-BR (default) or en locale files, and both files SHALL have identical key sets.
14. **[REQ-014]** WHEN the board page renders THEN the system SHALL have no axe-core violations for the WCAG 2.1 A/AA rules that run in jsdom.

## Edge Cases

- **WebKitGTK blank window on Wayland** - a known DMABUF renderer bug; the spike documents `WEBKIT_DISABLE_DMABUF_RENDERER=1` as the workaround if it appears.
- **Glass too slow on WebKitGTK** - the spike switches the default visual mode to flat (one constant and its test); the user accepted dropping blur for a borders-and-colors interface.
- **`prefers-reduced-transparency`** - glass panels fall back to the flat fill.
- **Local storage unavailable** - visual mode reading never throws; it falls back to env, then glass.
- **Empty column** - shows "Nenhuma vaga aqui." instead of an empty box.
- **Drop on the same column** - no IPC call.
- **Drop on Encerradas** - nothing moves until an outcome is chosen; cancel leaves the card where it was.
- **Moving out of Encerradas** - the outcome is cleared.
- **`move_card` failure** - the card stays in place and the board shows the error text.
- **Filters that match nothing** - every column shows its empty state; metrics show "—" where a rate or average has no data.
- **Narrower monitors** - columns keep a 300 px minimum and the board scrolls horizontally below about 2100 px of board width.

## Architecture Decisions

1. **Tauri 2 + Rust + React** - Chose Tauri over Electron and a local web server because it ships a ~10-15 MB binary, opens no local port (IPC instead of REST, so no DNS-rebinding/CSRF surface), and keeps one backend language for the future crawler, CLI runner and MCP. Discarded: Electron (~200 MB, crawler rewrite in TS anyway) and a Python local web app (open localhost API to harden).
2. **ts-rs for the IPC contract** - Chose ts-rs 12 (stable) over tauri-specta (still release candidate) to generate TypeScript types from the Rust domain during `cargo test`; command wrappers stay hand-written and tiny.
3. **`Store` trait from day one** - Board code depends only on the trait, so SQLite and Postgres backends drop in later without touching UI or commands.
4. **Mocks served by Rust** - Chose a Rust `MockStore` over frontend fixtures to exercise the IPC path and the move rules now.
5. **Feature-Sliced Design in `src/`** - `app`, `pages`, `widgets`, `features`, `entities`, `shared`, with `@/` alias.
6. **shadcn/ui on Tailwind 4 with Noir tokens** - shadcn primitives (Radix) for dialog, menus and slider accessibility; colors from the Terminal Noir wallpaper.
7. **Drag-and-drop with @dnd-kit/core, keyboard path through a menu** - Pointer drag on the whole card (6 px activation so clicks still open the dialog); every move is also reachable through the card's "Mover para" menu.
8. **i18n with i18next from the first screen** - pt-BR default, en complete; cheaper now than extracting strings later.
9. **Visual mode switch (`glass` | `flat`)** - one `data-visual` attribute on `<html>` drives the panel style, so the WebKitGTK spike outcome is a one-line default change.

## Grilling Log

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| 1 | Where the app lives | New repo `job-radar-desktop`; unrelated to Forja | User: separate personal product, open source |
| 2 | Server or local | Local-first Tauri binary; optional Postgres on the homelab | User wants zero servers by default, homelab as exception |
| 3 | Storage backends | SQLite default + Postgres optional behind `Store`; only the trait + mock now | Open-source users must run without a server |
| 4 | Mock source | Rust command over IPC | Validates Rust -> React path from the start |
| 5 | UI language | i18n now, pt-BR + en | Open source audience |
| 6 | WebKitGTK risk | Spike as the first UI task; drop blur for flat if needed | User accepted a simpler borders-and-colors UI |
| 7 | Dragging | Yes, plus keyboard alternative | Iron Law 5 (keyboard support) |
| 8 | Card dialog depth | Functional skeleton; CV generation disabled | Content design comes later |
| 9 | Filters and metrics | Client-side over mocks, metrics follow the filters | Retrospective by source/target needs filtered metrics |
| 10 | "Contratado" column | Removed; hiring is an outcome inside Encerradas | User decision on the Stitch review |
| 11 | Salary | Negotiated PJ and/or CLT, monthly BRL, shown from Candidatado on | User decision on the Stitch review |
| 12 | "Gerar CV customizado" | Only inside the dialog, never on the card | User decision on the Stitch review |

## Constraints

- Linux (Niri/Wayland, WebKitGTK 4.1) is the primary target; macOS/Windows must build but are not tested here.
- No real data, network access, database or AI CLI in this slice.
- Repository is public: no personal hostnames, IPs, paths or credentials in code or config.
- TypeScript 6 (scaffold default); pinned versions as of 2026-11-02.
- Tauri capabilities limited to core defaults plus the four window controls the title bar needs; strict CSP.

## Out of Scope

| Item | Reason |
|------|--------|
| SQLite and Postgres stores | Next plan; only the trait exists now |
| Crawler port to Rust, `crawl` and `mcp` modes | Later plans |
| Running AI CLIs, Gmail sync, CV generation | Later plans; buttons are disabled placeholders |
| Profile, Resumes, Preparation and Settings screens | Placeholders only |
| Notifications (Discord/Telegram) | Undecided, later |
| Language switcher UI | Locale files ship now; switching comes with Settings |
| Packaging, signing, auto-update, README | Release work, later |

## Test Strategy

- **Unit (Rust):** domain serialization, move rules, mock seed invariants and command validation with `cargo test`; ts-rs export runs in the same suite.
- **Unit/Component (web):** Vitest + Testing Library in jsdom for pure functions (visual mode, filters, metrics, grouping, formatting), hooks with a fake API, and components (shell, card, board, menus, dialog).
- **Accessibility:** axe-core over the rendered board page in jsdom (contrast checked manually, jsdom cannot compute it).
- **Manual:** WebKitGTK spike on Niri (glass vs flat), pointer drag-and-drop, window controls.

## Requirement Traceability

| ID | Description | Priority | Status |
|----|-------------|----------|--------|
| REQ-001 | Scaffold builds, checks pass, window opens | P1 | Pending |
| REQ-002 | Noir background with glass/flat visual modes | P1 | Pending |
| REQ-003 | IPC contract serialization + ts-rs bindings | P1 | Pending |
| REQ-004 | `list_board` via `Store` trait and `MockStore` | P1 | Pending |
| REQ-005 | `move_card` rules and validation | P1 | Pending |
| REQ-006 | Title bar and sidebar navigation with placeholders | P1 | Pending |
| REQ-007 | Six columns with counts and full card content | P1 | Pending |
| REQ-008 | Negotiated PJ/CLT salary display | P1 | Pending |
| REQ-009 | Outcome badge in Encerradas | P1 | Pending |
| REQ-010 | Search, filters and metrics | P1 | Pending |
| REQ-011 | Drag-and-drop and "Mover para" menu | P1 | Pending |
| REQ-012 | Card dialog skeleton | P1 | Pending |
| REQ-013 | pt-BR/en locales with identical keys | P1 | Pending |
| REQ-014 | No axe violations on the board page | P1 | Pending |
