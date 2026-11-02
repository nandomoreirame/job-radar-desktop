# Roadmap

What is left before job-radar-desktop becomes a **functional MVP**: an app you can run your real job search on, with
jobs flowing in, data that survives a restart, and AI help through the CLIs you already use. Everything after the MVP
line is wanted but not required for it.

## MVP progress

```text
██████░░░░░░░░░░░░░░  30%  (3 of 11 milestones, weighted by effort)
```

| #   | Milestone                                                                                                     |  Weight | Status                                                |
| --- | ------------------------------------------------------------------------------------------------------------- | ------: | ----------------------------------------------------- |
| 1   | Board prototype: Tauri + React shell, Rust domain, six columns, filters, metrics, i18n                        |      10 | Done                                                  |
| 2   | UI on shadcn Base UI and Dice UI, Kanban, command palette, job edit form with validated `update_card`         |      10 | Done (independent review and manual check pending)    |
| 3   | SQLite store: cards and edits survive a restart                                                               |      10 | Done (independent review and manual UI check pending) |
| 4   | Real jobs in: port the job-radar crawler to Rust (`job-radar crawl`) or import its output                     |      15 | Planned                                               |
| 5   | Add a job by hand ("+ Adicionar vaga")                                                                        |       5 | Planned                                               |
| 6   | Settings page: job sources, targets, score cut, visual mode, language                                         |       8 | Planned                                               |
| 7   | Profile page: the career source of truth the AI actions read from                                             |       8 | Planned                                               |
| 8   | AI actions through local CLIs (Claude Code, Codex, Grok): tailored CV first, with a tool allowlist per action |      15 | Planned                                               |
| 9   | Resumes page: review and approve generated CVs                                                                |       5 | Planned                                               |
| 10  | Preparation page: interview prep linked to each job                                                           |       6 | Planned                                               |
| 11  | Linux release: AppImage and deb from `pnpm tauri build`, update notes                                         |       8 | Planned                                               |
|     | **Total**                                                                                                     | **100** | **30 done**                                           |

The bar counts only milestones that are implemented and pass the test gates; partial work in a planned milestone counts
as zero. "Gerar CV customizado" and "Executar agora" already exist as buttons, but they only say "not implemented yet"
until milestones 4 and 8 land.

## After the MVP

- **Optional Postgres store** on a self-hosted server, behind the same `Store` trait, for running the crawler 24/7 off
  the desktop.
- **MCP server mode** (`job-radar mcp`, stdio) so AI CLIs can read and update the board.
- **Gmail sync** of recruiter replies through the CLIs' MCP servers, read-only.
- **Coverage review and interview prep** as AI actions, after the tailored CV.
- **Notifications** for new jobs (Telegram or Discord, opt-in).
- **Retrospective metrics**: response rate by source, target and CV version over time.
- **Language switcher** in Settings (the pt-BR and en locales already ship).
- **macOS and Windows builds**.

## How the roadmap moves

Each milestone becomes a spec and a plan under [docs/specs](docs/specs) and [docs/plans](docs/plans) before any code,
and it is marked done only when its tests, type-check, lint, build and the plan's checks pass. Update the bar and the
table in the same commit that closes a milestone.
