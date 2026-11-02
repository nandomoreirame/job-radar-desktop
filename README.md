# job-radar-desktop

Local-first desktop tracker for a job search: board, filters and AI-assisted applications (Tauri + React).

[Português](README.pt-br.md)

![job-radar-desktop board: six columns from Inbox to Closed with job cards, filters, metrics and the collapsible sidebar, over the Terminal Noir skyline](docs/images/board.png)

> **Status: pre-alpha.** Cards are saved in a local SQLite file and the crawler searches Gupy and
> Solides; there is no AI integration yet; see [Roadmap](#roadmap).

## Why

Developers looking for a job collect postings from a dozen sources, then lose track of what
happened next: which CV went where, who replied, which interview is next, which source never
answers. A spreadsheet or a generic Kanban board holds the list but not the context, and SaaS
trackers keep your CV and your applications on someone else's server. job-radar-desktop keeps the
whole pipeline on your machine, from inbox to hired or closed, and is built to drive the AI CLIs
you already use (Claude Code, Codex, Grok) instead of shipping its own API keys.

It pairs with [job-radar](https://github.com/nandomoreirame/job-radar), the crawler that finds and
scores the postings.

## Features

- See every application in six columns: Inbox, Triage, Applied, In process, Offer, Closed.
- Move a card by dragging it or from its menu, and record why it closed (hired, rejected, discarded, no response).
- Filter by source, target role and minimum score; the metrics strip (applications this week, response rate, average days to a reply) follows the filters.
- Open a card to edit your side of the application: salary expectation (PJ and/or CLT), recruiter contact, links, next action and notes.
- Use it in Brazilian Portuguese (default) or English.

## Quick start

Prerequisites:

- Rust 1.85 or newer (edition 2024)
- Node.js 24 and pnpm 11
- The Tauri 2 system dependencies for your OS: <https://tauri.app/start/prerequisites/> (on Linux this includes `webkit2gtk-4.1`)

```bash
git clone https://github.com/nandomoreirame/job-radar-desktop.git
cd job-radar-desktop
pnpm install
pnpm tauri dev
```

A frameless, maximized window opens with an empty board; see Usage to load jobs.

On Wayland, if the window stays blank, run `WEBKIT_DISABLE_DMABUF_RENDERER=1 pnpm tauri dev`
(a WebKitGTK renderer issue).

## Usage

| Action | How |
|---|---|
| Edit a job | Click the card title (or Tab to it and press Enter), change the fields and Save |
| Move a job | Drag the card, or open its "..." menu and pick a column |
| Close a job | Drop it on Closed and choose the outcome, or use "Close as" in its menu |
| Filter | Source, target and minimum score in the toolbar |
| Find new jobs | "Executar agora" (Run now) in the sidebar; jobs below the score cut land on Discarded |
| Load a crawler's history | `job-radar-desktop import --seen seen.db --candidates candidates.json` (safe to repeat) |
| Review discarded jobs | Open Discarded in the sidebar, then move a job to the Inbox or delete it for good |
| Switch the visual style | Set `VITE_VISUAL_MODE=flat` (solid panels) or `glass` (default, translucent panels) |

Everything is saved in SQLite at `~/.local/share/dev.nandomoreira.jobradar/job-radar.db` (development builds use `dev.nandomoreira.jobradar-local`). Set `JOB_RADAR_DB` to use another file. Imported jobs have no score until the crawler reads their page.
An imported job shows "No score" until the crawler reads its page; a minimum score never hides it.

## Configuration

The crawler reads `crawler.json` from `~/.local/share/dev.nandomoreira.jobradar/` (development builds use `dev.nandomoreira.jobradar-local`). Without it, "Run now" says what to fix and makes no request.

```json
{
  "profile_path": "/path/to/cv.md",
  "min_score": 70
}
```

| Field | Default | Meaning |
|---|---|---|
| `profile_path` | required | Your resume in Markdown; it needs a `## Stack técnica` table, the source of your skills |
| `min_score` | `70` | Below this, a new job goes to Discarded |
| `min_skills` | `3` | Fewer skills from your profile than this rejects the job |
| `min_salary_pj`, `min_salary_clt` | `0` (off) | Salary floors; a job that states no salary passes |
| `commute_cities` | empty | Cities where you accept hybrid work; on-site jobs are always rejected |
| `max_job_age_days` | `0` (off) | Rejects older jobs |
| `blocked_companies` | empty | Companies to reject |

An unknown field or an invalid value refuses the run and names the field.

## Development

| Task | Command |
|---|---|
| Web tests (Vitest, jsdom) | `pnpm test` |
| Rust tests | `cd src-tauri && cargo test` |
| Type-check | `pnpm typecheck` |
| Lint | `pnpm lint` |
| Production web build | `pnpm build` |
| Desktop bundle | `pnpm tauri build` |

`cargo test`, run from `src-tauri/`, also regenerates the TypeScript types in `src/shared/bindings` from the Rust domain
(ts-rs), so the React side never redeclares them.

Layout: `src-tauri/` holds the Rust domain, the `Store` trait and the IPC commands; `src/` is the
React app in Feature-Sliced Design (`app`, `pages`, `widgets`, `features`, `entities`, `shared`).
Specs and implementation plans live in [docs/](docs/).

## Roadmap

MVP progress and every planned milestone live in [ROADMAP.md](ROADMAP.md). The main ones:

- SQLite store by default, with an optional Postgres on a self-hosted server.
- Port of the job-radar crawler to Rust, runnable headless from a timer (`job-radar crawl`).
- AI actions through your local CLIs: tailored CV, coverage review, interview prep.
- Gmail sync of recruiter replies through the CLIs' MCP servers.
- An MCP server mode (`job-radar mcp`) so AI CLIs can read and update the board.

## Security

- **Requests only from Rust, only when you run the crawler.** It contacts `portal.gupy.io`, company
  `*.gupy.io` job pages, `vagas.solides.com.br`, and each host's `robots.txt`. It sends a User-Agent
  and the page path, never your profile, notes or any credential. It honors `robots.txt`, waits at
  least 1 s per host and bounds time and body size. The webview's content security policy still
  allows only the Tauri IPC channel (`connect-src ipc: http://ipc.localhost`).
- **No open port.** The frontend talks to Rust through Tauri IPC, not a local web server. Only
  `pnpm tauri dev` starts Vite on `localhost:1420`, for development.
- **Minimal permissions.** The window gets Tauri's core defaults plus minimize, maximize, close
  and drag for the custom title bar; no shell, file system or HTTP plugins are enabled. The opener plugin is used only from Rust, for a link already stored and checked; the webview cannot open a URL of its own.
- **IPC input is validated.** Card ids are bounded (1 to 64 characters) and stage/outcome pairs
  are checked in Rust before any data changes.
- **Import reads local files only,** read-only, and treats their content as untrusted.
- **Local data only.** Jobs live in a SQLite file in your data directory, created readable only by you (0600); the browser storage keeps only the visual mode.

Planned features change this surface: running AI CLIs and reading Gmail will be opt-in, with
read-only Gmail access and an explicit tool allowlist per action. To report a vulnerability,
open a private security advisory on this repository.

## Contributing

Issues and pull requests are welcome. Run `pnpm test`, `pnpm typecheck`, `pnpm lint` and the
Rust tests before opening a PR.

## License

[MIT](LICENSE) © 2026 Fernando Moreira Costa
