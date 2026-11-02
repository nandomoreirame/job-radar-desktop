# Plan: Board UI Migration (shadcn Base UI + Dice UI)

> **Spec:** docs/specs/board-ui-migration/SPEC.md

**Goal:** Move every UI primitive to shadcn base-nova (Base UI) and Dice UI, with a Kanban board, responsive job dialog, status badges, a Ctrl/Cmd+K palette, inline Field filters and a collapsible icon sidebar.
**Architecture:** shadcn is re-pointed to the `base-nova` style and the `@diceui` registry; generated primitives live in `src/shared/ui`. The board keeps the store as source of truth: a `useKanbanColumns` hook holds the local columns Dice UI Kanban mutates while dragging, resolves the drop against the drag origin and restores on cancel. FSD layers and the Rust side stay as they are.
**Tech Stack:** shadcn 4.23 (base-nova), @base-ui/react 1.9, Dice UI (kanban, responsive-dialog, status), @dnd-kit/core 6.3 + sortable 10 + modifiers 9, cmdk 1.1, React 19.3, Vitest 5, Testing Library, axe-core.
**Total Tasks:** 10
**Estimated Complexity:** large
**Lessons applied:** none

Conventions for every task:
- Commands run from the repo root (`~/dev/projects/job-radar-desktop`).
- Use `pnpm <script>` or `./node_modules/.bin/<tool>`, never a bare `tsc`/`eslint`.
- UI strings only through i18n keys; code and comments in English.
- shadcn/Dice UI files under `src/shared/ui` are generated: edit them only where a step says so.

Batches: B1 = Tasks 1-2 (Base UI switch, locales), B2 = Tasks 3-6 (status, kanban model, kanban board, responsive dialog), B3 = Tasks 7-9 (filters, palette, sidebar), B4 = Task 10 (a11y and gates).

---

## Checks

**Run web**: `pnpm vitest run --reporter=verbose`

- C1 (REQ-001): radix-ui is gone from the dependencies. Proof: run web | ✓ primitives keep radix-ui out of the dependencies
- C2 (REQ-001): no file under src imports radix. Proof: run web | ✓ primitives import no radix primitive under src
- C3 (REQ-001): the app still builds on Base UI. Proof: `pnpm build`
- C4 (REQ-002): every card has a labeled drag handle. Proof: run web | ✓ Board labels each drag handle
- C5 (REQ-003): a drop on Encerradas asks for the outcome. Proof: run web | ✓ useKanbanColumns asks for an outcome when dropped on Encerradas
- C6 (REQ-003): a cancelled drag goes back to the store columns. Proof: run web | ✓ useKanbanColumns restores the store columns
- C7 (REQ-003): a reorder inside a column does not move the card. Proof: run web | ✓ useKanbanColumns ignores a reorder inside the column
- C8 (REQ-004): narration names the column under a hovered card. Proof: run web | ✓ buildAnnouncements names the column of the card hovered
- C9 (REQ-005): the job details render as the dialog variant on desktop. Proof: run web | ✓ JobDialog renders the dialog variant on desktop
- C10 (REQ-006): outcomes map to status variants. Proof: run web | ✓ JobCard maps the outcome to a status variant
- C11 (REQ-007): the palette matches without accents. Proof: run web | ✓ JobCommand finds a job without accents
- C12 (REQ-007): Ctrl+K opens a job. Proof: run web | ✓ BoardPage opens a job from the command palette
- C13 (REQ-008): the score field is clamped. Proof: run web | ✓ Toolbar clamps the minimum score
- C14 (REQ-008): the unread checkbox filters. Proof: run web | ✓ Toolbar filters unread only
- C15 (REQ-009): the sidebar starts collapsed and remembers. Proof: run web | ✓ AppShell starts collapsed and remembers the choice
- C16 (REQ-010): the board passes axe with the palette open. Proof: run web | ✓ BoardPage has no axe violations with the palette open
- C17 (REQ-010): locales share the same keys. Proof: run web | ✓ locales have the same keys in pt-BR and en

---

### Task 1: Switch shadcn to base-nova, add the Dice UI registry, drop radix-ui

**Requirement:** REQ-001
**Files:**
- Modify: `components.json`
- Modify: `package.json`
- Modify: `src/shared/ui/button.tsx`
- Modify: `src/shared/ui/badge.tsx`
- Modify: `src/shared/ui/dialog.tsx`
- Modify: `src/shared/ui/dropdown-menu.tsx`
- Modify: `src/shared/ui/input.tsx`
- Create: `src/shared/ui/field.tsx`
- Create: `src/shared/ui/checkbox.tsx`
- Create: `src/shared/ui/select.tsx`
- Create: `src/shared/ui/command.tsx`
- Create: `src/shared/ui/sidebar.tsx`
- Create: `src/shared/ui/tooltip.tsx`
- Create: `src/shared/ui/kanban.tsx`
- Create: `src/shared/ui/responsive-dialog.tsx`
- Create: `src/shared/ui/status.tsx`
- Create: `src/shared/lib/hooks/use-mobile.ts`
- Modify: `src/features/move-card/ui/MoveMenu.tsx`
- Modify: `src/features/filter-board/ui/Toolbar.tsx`
- Modify: `src/entities/job/ui/JobDialog.tsx`
- Modify: `src/test/setup.ts`
- Test: `src/shared/ui/primitives.test.ts`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, consequences-reviewer, accessibility-guardrails

**Step 1: Write the failing test**
```ts
// src/shared/ui/primitives.test.ts
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
  dependencies: Record<string, string>;
};

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(ts|tsx)$/.test(name) ? [path] : [];
  });
}

describe("primitives", () => {
  it("keep radix-ui out of the dependencies", () => {
    expect(pkg.dependencies).not.toHaveProperty("radix-ui");
    expect(pkg.dependencies).toHaveProperty("@base-ui/react");
  });

  it("import no radix primitive under src", () => {
    const offenders = sourceFiles(join(root, "src")).filter((file) =>
      /from ["'](radix-ui|@radix-ui\/)/.test(readFileSync(file, "utf8")),
    );
    expect(offenders).toEqual([]);
  });
});
```
(Vitest prints the `it` names under the describe, so the checks read `primitives keep radix-ui out of the dependencies` and `primitives import no radix primitive under src`.)

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/shared/ui/primitives.test.ts`
Expected: FAIL with "expected { …(15) } to not have property "radix-ui""

**Step 3: Write minimal implementation**

Re-point shadcn and register Dice UI (`components.json`, only these two changes):
```json
  "style": "base-nova",
```
```json
  "registries": {
    "@diceui": "https://diceui.com/r/{name}.json"
  }
```

Regenerate the existing primitives on Base UI, add the new ones, then drop Radix:
```bash
pnpm dlx shadcn@4.23.0 add button badge dialog dropdown-menu input --overwrite --yes
pnpm dlx shadcn@4.23.0 add field checkbox select command sidebar tooltip --yes
pnpm dlx shadcn@4.23.0 add @diceui/kanban @diceui/responsive-dialog @diceui/status --overwrite --yes
pnpm remove radix-ui
pnpm audit --prod
```
Then:
- If any generated file imports `cn` from `"cn"` or `@/registry/...`, fix it to the aliases in `components.json` (`@/shared/lib/utils`, `@/shared/ui/*`, `@/shared/lib/hooks/*`).
- If the CLI rewrote `src/shared/styles/theme.css`, restore the Terminal Noir values (compare with `git show HEAD:src/shared/styles/theme.css`); keep any new token it added.
- If a second `use-mobile` file appeared (shadcn and Dice UI both ship one), keep one at `src/shared/lib/hooks/use-mobile.ts` and point every import at it.

Base UI uses `render` instead of `asChild`, `onClick` instead of `onSelect` on menu items, and requires labels inside groups.

`src/features/move-card/ui/MoveMenu.tsx` (full replacement):
```tsx
import { MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { OUTCOMES, STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";

type Props = { card: BoardCard; onMove: (stage: Stage, outcome: Outcome | null) => void };

/** Keyboard path for every move, and the only way to close with an explicit outcome. */
export function MoveMenu({ card, onMove }: Props) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button variant="ghost" size="icon" className="size-7" aria-label={t("card.menu", { title: card.title })} />
        }
      >
        <MoreHorizontal aria-hidden="true" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuGroup>
          <DropdownMenuLabel>{t("move.to")}</DropdownMenuLabel>
          {STAGES.filter((stage) => stage !== "closed" && stage !== card.stage).map((stage) => (
            <DropdownMenuItem key={stage} onClick={() => onMove(stage, null)}>
              {t(`stage.${stage}`)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuLabel>{t("move.closeAs")}</DropdownMenuLabel>
          {OUTCOMES.filter((outcome) => outcome !== card.outcome).map((outcome) => (
            <DropdownMenuItem key={outcome} onClick={() => onMove("closed", outcome)}>
              {t(`outcome.${outcome}`)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

`src/features/filter-board/ui/Toolbar.tsx` (bridge until Task 7 rewrites it): in both dropdowns replace
```tsx
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="h-9">
```
with
```tsx
        <DropdownMenuTrigger render={<Button variant="outline" className="h-9" />}>
```
delete the matching `</Button>` line before each `</DropdownMenuTrigger>`, and delete both `onSelect={(event) => event.preventDefault()}` lines.

`src/entities/job/ui/JobDialog.tsx`: replace
```tsx
            <DialogClose asChild>
              <Button variant="ghost">{t("dialog.close")}</Button>
            </DialogClose>
```
with
```tsx
            <DialogClose render={<Button variant="ghost" />}>{t("dialog.close")}</DialogClose>
```

`src/test/setup.ts`: change the comment above the polyfills to `// Base UI, cmdk and dnd-kit call browser APIs that jsdom does not implement.` and append:
```ts
// useIsMobile (ResponsiveDialog, Sidebar) reads matchMedia; jsdom has none.
window.matchMedia ??= ((query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addEventListener: () => {},
  removeEventListener: () => {},
  addListener: () => {},
  removeListener: () => {},
  dispatchEvent: () => false,
})) as unknown as typeof window.matchMedia;
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint && pnpm build`
Expected: PASS (both `primitives` tests and every existing suite, including MoveMenu, Toolbar, JobDialog and BoardPage)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Locale strings for drag, palette, filters and sidebar

**Requirement:** REQ-010, REQ-004
**Files:**
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/shared/locales/locales.test.ts`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Add to the `describe("locales", ...)` block in `src/shared/locales/locales.test.ts`:
```ts
  it("carry the migration strings in both languages", () => {
    const needed = [
      "card.drag",
      "dnd.instructions",
      "command.title",
      "command.description",
      "command.placeholder",
      "command.empty",
      "command.group",
      "command.open",
      "command.shortcut",
      "filters.label",
      "filters.unreadOnly",
      "sidebar.toggle",
    ];
    for (const lng of ["pt-BR", "en"]) {
      const t = i18n.getFixedT(lng);
      for (const key of needed) expect(t(key), `${lng}:${key}`).not.toBe(key);
    }
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/shared/locales/locales.test.ts`
Expected: FAIL with "pt-BR:card.drag: expected 'card.drag' not to be 'card.drag'"

**Step 3: Write minimal implementation**
```bash
f=src/shared/locales/pt-BR.json; tmp=$(mktemp)
jq '.card.drag = "Arrastar {{title}}"
  | .dnd.instructions = "Para pegar uma vaga, pressione espaço ou Enter. Use as setas para movê-la e pressione espaço ou Enter de novo para soltar, ou Esc para cancelar."
  | .command = {title: "Buscar vaga", description: "Busque por cargo ou empresa e abra os detalhes.",
                placeholder: "Cargo ou empresa…", empty: "Nenhuma vaga encontrada.", group: "Vagas",
                open: "Buscar vaga", shortcut: "Ctrl K"}
  | .filters.label = "Filtros" | .filters.unreadOnly = "Somente não lidas"
  | .sidebar = {toggle: "Alternar barra lateral"}' "$f" > "$tmp" && [ -s "$tmp" ] && mv "$tmp" "$f"

f=src/shared/locales/en.json; tmp=$(mktemp)
jq '.card.drag = "Drag {{title}}"
  | .dnd.instructions = "To pick up a job, press space or Enter. Use the arrow keys to move it, press space or Enter again to drop it, or Escape to cancel."
  | .command = {title: "Search jobs", description: "Search by title or company and open the details.",
                placeholder: "Title or company…", empty: "No jobs found.", group: "Jobs",
                open: "Search jobs", shortcut: "Ctrl K"}
  | .filters.label = "Filters" | .filters.unreadOnly = "Unread only"
  | .sidebar = {toggle: "Toggle sidebar"}' "$f" > "$tmp" && [ -s "$tmp" ] && mv "$tmp" "$f"
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose`
Expected: PASS (`✓ locales carry the migration strings in both languages`, `✓ locales have the same keys in pt-BR and en`)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: Dice UI Status on job cards, recolored to Terminal Noir

**Requirement:** REQ-006
**Files:**
- Modify: `src/shared/ui/status.tsx`
- Modify: `src/entities/job/model/format.ts`
- Modify: `src/entities/job/ui/JobCard.tsx`
- Test: `src/entities/job/model/format.test.ts`
- Test: `src/entities/job/ui/JobCard.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Append to `src/entities/job/model/format.test.ts`:
```ts
import { outcomeVariant } from "./format";

describe("outcomeVariant", () => {
  it("maps each outcome to a status variant", () => {
    expect(outcomeVariant("hired")).toBe("success");
    expect(outcomeVariant("rejected")).toBe("error");
    expect(outcomeVariant("discarded")).toBe("default");
    expect(outcomeVariant("no_response")).toBe("default");
  });
});
```
(Merge the import into the existing import line from `./format`.)

Append inside `describe("JobCard", ...)` in `src/entities/job/ui/JobCard.test.tsx`:
```tsx
  it("maps the outcome to a status variant", () => {
    render(<JobCard card={makeCard({ stage: "closed", outcome: "rejected" })} onOpen={() => {}} />);
    expect(screen.getByText("Recusada").closest('[data-slot="status"]')).toHaveAttribute("data-variant", "error");
  });

  it("shows the interview step as a warning status", () => {
    render(<JobCard card={makeCard({ stage: "in_process", processStep: "Técnica" })} onOpen={() => {}} />);
    expect(screen.getByText("Técnica").closest('[data-slot="status"]')).toHaveAttribute("data-variant", "warning");
  });

  it("shows the running CLI as an info status with an indicator", () => {
    render(<JobCard card={makeCard({ aiRunning: "codex" })} onOpen={() => {}} />);
    const status = screen.getByRole("status");
    expect(status).toHaveAttribute("data-variant", "info");
    expect(status.querySelector('[data-slot="status-indicator"]')).not.toBeNull();
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/entities/job`
Expected: FAIL with "outcomeVariant is not a function" (or "is not exported")

**Step 3: Write minimal implementation**

`src/shared/ui/status.tsx`: replace the `variants.variant` object inside `statusVariants` with the Noir palette (leave the rest of the file as generated):
```ts
    variant: {
      default: "border-transparent bg-white/5 text-noir-muted **:data-[slot=status-indicator]:bg-noir-muted",
      success: "border-noir-teal/30 bg-noir-teal/10 text-noir-teal **:data-[slot=status-indicator]:bg-noir-teal",
      error: "border-noir-red/30 bg-noir-red/10 text-noir-red **:data-[slot=status-indicator]:bg-noir-red",
      warning: "border-noir-amber/40 bg-noir-amber/10 text-noir-amber **:data-[slot=status-indicator]:bg-noir-amber",
      info: "border-noir-cream/30 bg-noir-cream/10 text-noir-cream **:data-[slot=status-indicator]:bg-noir-cream",
    },
```

Append to `src/entities/job/model/format.ts`:
```ts
import type { Outcome } from "@/shared/bindings/Outcome";

export type StatusVariant = "default" | "success" | "error" | "warning" | "info";

export function outcomeVariant(outcome: Outcome): StatusVariant {
  if (outcome === "hired") return "success";
  if (outcome === "rejected") return "error";
  return "default";
}
```
(Move the new import to the top of the file next to the `Salary` import.)

`src/entities/job/ui/JobCard.tsx`: swap the badge-based outcome, process step and AI indicator for Status, and lay the actions out in a row:
1. Imports: remove `Loader2` from the lucide import and the `OUTCOME_TONE` constant; add
```tsx
import { Status, StatusIndicator, StatusLabel } from "@/shared/ui/status";
import { outcomeVariant, scoreTone } from "../model/format";
```
(replacing the old `import { scoreTone } from "../model/format";`).
2. Replace the outcome badge block with:
```tsx
      {card.outcome && (
        <Status variant={outcomeVariant(card.outcome)}>
          <StatusLabel>{t(`outcome.${card.outcome}`)}</StatusLabel>
        </Status>
      )}
```
3. Replace the process step `Badge` with:
```tsx
        {card.processStep && (
          <Status variant="warning" className="font-mono uppercase">
            <StatusLabel>{card.processStep}</StatusLabel>
          </Status>
        )}
```
4. Replace the AI-running `<p role="status">…</p>` with:
```tsx
        <Status role="status" variant="info" className="w-full">
          <StatusIndicator />
          <StatusLabel>{t("card.aiRunning", { cli: capitalize(card.aiRunning) })}</StatusLabel>
        </Status>
```
5. Replace the actions wrapper with:
```tsx
      {actions && <div className="relative z-10 flex items-center gap-1 self-end">{actions}</div>}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ outcomeVariant maps each outcome to a status variant`, `✓ JobCard maps the outcome to a status variant`, the two other new JobCard tests, and the existing `shows the AI running status`)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: Kanban column model, drop resolution and translated narration

**Requirement:** REQ-003, REQ-004
**Files:**
- Create: `src/widgets/board/model/kanban.ts`
- Create: `src/widgets/board/model/use-kanban-columns.ts`
- Modify: `src/widgets/board/model/announcements.ts`
- Modify: `src/widgets/board/Board.tsx`
- Test: `src/widgets/board/model/kanban.test.ts`
- Test: `src/widgets/board/model/use-kanban-columns.test.ts`
- Test: `src/widgets/board/model/announcements.test.ts`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: business-logic-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

`src/widgets/board/model/kanban.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { columnOf, resolveKanbanDrop } from "./kanban";
import { groupByStage } from "./group";

const columns = groupByStage([makeCard({ id: "a" }), makeCard({ id: "b", stage: "offer" })]);

describe("columnOf", () => {
  it("finds the stage holding a card", () => {
    expect(columnOf(columns, "b")).toBe("offer");
  });

  it("returns undefined for an unknown id", () => {
    expect(columnOf(columns, "zzz")).toBeUndefined();
  });
});

describe("resolveKanbanDrop", () => {
  it("moves to another open stage", () => {
    expect(resolveKanbanDrop("a", "inbox", "triage")).toEqual({ id: "a", kind: "move", stage: "triage" });
  });

  it("asks for an outcome before closing", () => {
    expect(resolveKanbanDrop("a", "inbox", "closed")).toEqual({ id: "a", kind: "close" });
  });

  it("ignores the same column and a lost card", () => {
    expect(resolveKanbanDrop("a", "inbox", "inbox")).toBeNull();
    expect(resolveKanbanDrop("a", "inbox", undefined)).toBeNull();
  });
});
```

`src/widgets/board/model/use-kanban-columns.test.ts`:
```ts
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { makeCard } from "@/test/factories";
import { useKanbanColumns } from "./use-kanban-columns";

const a = makeCard({ id: "a" });
const b = makeCard({ id: "b" });
const ids = (list: BoardCard[]) => list.map((card) => card.id);

function dragTo(result: { current: ReturnType<typeof useKanbanColumns> }, stage: "triage" | "closed") {
  const { columns } = result.current;
  act(() =>
    result.current.setColumns({ ...columns, inbox: columns.inbox.filter((c) => c.id !== "a"), [stage]: [a] }),
  );
}

describe("useKanbanColumns", () => {
  it("starts from the store cards", () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    expect(ids(result.current.columns.inbox)).toEqual(["a", "b"]);
  });

  it("resolves a cross-column drag into a move", () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin("a"));
    dragTo(result, "triage");
    expect(result.current.finish()).toEqual({ id: "a", kind: "move", stage: "triage" });
  });

  it("asks for an outcome when dropped on Encerradas", () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin("a"));
    dragTo(result, "closed");
    expect(result.current.finish()).toEqual({ id: "a", kind: "close" });
  });

  it("ignores a reorder inside the column", () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin("a"));
    act(() => result.current.setColumns({ ...result.current.columns, inbox: [b, a] }));
    expect(result.current.finish()).toBeNull();
  });

  it("restores the store columns", () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin("a"));
    dragTo(result, "closed");
    act(() => result.current.restore());
    expect(ids(result.current.columns.inbox)).toEqual(["a", "b"]);
    expect(result.current.columns.closed).toEqual([]);
  });

  it("resyncs when the store cards change", () => {
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [a, b] },
    });
    rerender({ cards: [a, { ...b, stage: "offer" as const }] });
    expect(ids(result.current.columns.offer)).toEqual(["b"]);
  });

  it("finishes nothing without a drag in progress", () => {
    const { result } = renderHook(() => useKanbanColumns([a]));
    expect(result.current.finish()).toBeNull();
  });
});
```

`src/widgets/board/model/announcements.test.ts` (full replacement):
```ts
import type { Announcements } from "@dnd-kit/core";
import { describe, expect, it } from "vitest";
import i18n from "@/app/i18n";
import { makeCard } from "@/test/factories";
import { buildAnnouncements } from "./announcements";

const t = i18n.getFixedT("pt-BR");
const cards = [makeCard({ id: "a", title: "Alpha Engineer" }), makeCard({ id: "b", stage: "offer" })];
const columnOfCard = (id: unknown) => cards.find((card) => card.id === id)?.stage;
const announcements = buildAnnouncements(t, cards, columnOfCard);

type Event = Parameters<NonNullable<Announcements["onDragEnd"]>>[0];
const event = (active: string, over?: string) =>
  ({ active: { id: active }, over: over ? { id: over } : null }) as unknown as Event;

describe("buildAnnouncements", () => {
  it("names the card picked up", () => {
    expect(announcements.onDragStart?.(event("a"))).toBe("Alpha Engineer selecionada.");
  });

  it("names the column hovered", () => {
    expect(announcements.onDragOver?.(event("a", "triage"))).toBe("Alpha Engineer sobre Triagem.");
  });

  it("names the column of the card hovered", () => {
    expect(announcements.onDragOver?.(event("a", "b"))).toBe("Alpha Engineer sobre Oferta.");
  });

  it("stays quiet when hovering nothing", () => {
    expect(announcements.onDragOver?.(event("a"))).toBeUndefined();
  });

  it("reports where the card landed", () => {
    expect(announcements.onDragEnd?.(event("a", "offer"))).toBe("Alpha Engineer solta em Oferta.");
  });

  it("reports a cancelled drop", () => {
    expect(announcements.onDragEnd?.(event("a"))).toBe("Movimento de Alpha Engineer cancelado.");
    expect(announcements.onDragCancel?.(event("a"))).toBe("Movimento de Alpha Engineer cancelado.");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/widgets/board/model`
Expected: FAIL with "Failed to resolve import "./kanban"" and "Failed to resolve import "./use-kanban-columns""

**Step 3: Write minimal implementation**

`src/widgets/board/model/kanban.ts`:
```ts
import type { UniqueIdentifier } from "@dnd-kit/core";
import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

export type Columns = Record<Stage, BoardCard[]>;

/** What a finished drag means for the store. */
export type KanbanDrop =
  | { id: string; kind: "move"; stage: Stage }
  | { id: string; kind: "close" }
  | null;

export function columnOf(columns: Columns, id: UniqueIdentifier): Stage | undefined {
  return STAGES.find((stage) => columns[stage].some((card) => card.id === id));
}

/**
 * Dice UI Kanban moves the card between columns while dragging, so the drop is
 * the column holding the card at the end compared with where it started.
 */
export function resolveKanbanDrop(id: string, origin: Stage, final: Stage | undefined): KanbanDrop {
  if (!final || final === origin) return null;
  return final === "closed" ? { id, kind: "close" } : { id, kind: "move", stage: final };
}
```

`src/widgets/board/model/use-kanban-columns.ts`:
```ts
import { useCallback, useEffect, useRef, useState } from "react";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { groupByStage } from "./group";
import { columnOf, resolveKanbanDrop, type Columns, type KanbanDrop } from "./kanban";

/**
 * Local columns for Dice UI Kanban. The store stays the source of truth: any
 * new `cards` array resyncs the columns, and `restore` drops local changes.
 */
export function useKanbanColumns(cards: BoardCard[]) {
  const [columns, setColumnsState] = useState<Columns>(() => groupByStage(cards));
  const columnsRef = useRef(columns);
  const cardsRef = useRef(cards);
  const origin = useRef<{ id: string; stage: NonNullable<ReturnType<typeof columnOf>> } | null>(null);

  const setColumns = useCallback((next: Columns) => {
    columnsRef.current = next;
    setColumnsState(next);
  }, []);

  useEffect(() => {
    cardsRef.current = cards;
    setColumns(groupByStage(cards));
  }, [cards, setColumns]);

  const begin = useCallback((id: string) => {
    const stage = columnOf(columnsRef.current, id);
    origin.current = stage ? { id, stage } : null;
  }, []);

  const finish = useCallback((): KanbanDrop => {
    const start = origin.current;
    origin.current = null;
    if (!start) return null;
    return resolveKanbanDrop(start.id, start.stage, columnOf(columnsRef.current, start.id));
  }, []);

  const restore = useCallback(() => setColumns(groupByStage(cardsRef.current)), [setColumns]);

  return { columns, setColumns, begin, finish, restore };
}
```

`src/widgets/board/model/announcements.ts` (full replacement):
```ts
import type { Announcements, UniqueIdentifier } from "@dnd-kit/core";
import type { TFunction } from "i18next";
import { isStage } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

/**
 * Screen-reader narration for Kanban drags, in the active locale. Over a card,
 * Kanban reports the card id, so `columnOfCard` maps it to its column.
 */
export function buildAnnouncements(
  t: TFunction,
  cards: BoardCard[],
  columnOfCard: (id: UniqueIdentifier) => Stage | undefined,
): Announcements {
  const title = (id: UniqueIdentifier) => cards.find((card) => card.id === id)?.title ?? "";
  const column = (id: UniqueIdentifier) => {
    const stage = isStage(id) ? id : columnOfCard(id);
    return stage ? t(`stage.${stage}`) : "";
  };
  return {
    onDragStart: ({ active }) => t("dnd.picked", { title: title(active.id) }),
    onDragOver: ({ active, over }) =>
      over ? t("dnd.over", { title: title(active.id), column: column(over.id) }) : undefined,
    onDragEnd: ({ active, over }) =>
      over
        ? t("dnd.dropped", { title: title(active.id), column: column(over.id) })
        : t("dnd.cancelled", { title: title(active.id) }),
    onDragCancel: ({ active }) => t("dnd.cancelled", { title: title(active.id) }),
  };
}
```

Keep `src/widgets/board/Board.tsx` compiling until Task 5 replaces it: change its call to
`buildAnnouncements(t, cards, () => undefined)`.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ useKanbanColumns asks for an outcome when dropped on Encerradas`, `✓ buildAnnouncements names the column of the card hovered`, every existing Board test)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: Board on Dice UI Kanban with a labeled drag handle

**Requirement:** REQ-002, REQ-003, REQ-004
**Files:**
- Modify: `src/widgets/board/Board.tsx`
- Modify: `src/widgets/board/BoardColumn.tsx`
- Modify: `src/entities/job/model/use-board.ts`
- Test: `src/widgets/board/Board.test.tsx`
- Test: `src/entities/job/model/use-board.test.ts`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: business-logic-reviewer, type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Append inside `describe("Board", ...)` in `src/widgets/board/Board.test.tsx`:
```tsx
  it("labels each drag handle", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    expect(screen.getByRole("button", { name: "Arrastar Alpha Engineer" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Arrastar Gamma Lead" })).toBeInTheDocument();
  });

  it("keeps the move menu next to the handle", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Gamma Lead" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Em processo" }));
    expect(onMove).toHaveBeenCalledWith("c", "in_process", null);
  });
```

Append inside `describe("useBoard", ...)` in `src/entities/job/model/use-board.test.ts`:
```ts
  it("hands out a fresh list after a failed move so views resync", async () => {
    const api = fakeApi([makeCard()]);
    const { result } = renderHook(() => useBoard(api));
    await waitFor(() => expect(result.current.status).toBe("ready"));
    const before = result.current.cards;
    await act(() => result.current.move("missing", "triage", null));
    expect(result.current.cards).not.toBe(before);
    expect(result.current.cards).toEqual(before);
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/widgets/board/Board.test.tsx src/entities/job/model/use-board.test.ts`
Expected: FAIL with "Unable to find an accessible element with the role "button" and name "Arrastar Alpha Engineer"" and "expected [ …(1) ] not to be [ …(1) ]"

**Step 3: Write minimal implementation**

`src/entities/job/model/use-board.ts`: in the `catch` of `move`, after `setMoveError(true);` add
```ts
        // A new array identity makes views holding optimistic local state
        // (the Kanban columns) resync to the unchanged store data.
        setCards((list) => [...list]);
```

`src/widgets/board/BoardColumn.tsx` (full replacement):
```tsx
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { JobCard } from "@/entities/job/ui/JobCard";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";
import { cn } from "@/shared/lib/utils";
import { KanbanColumn, KanbanItem } from "@/shared/ui/kanban";

type Props = {
  stage: Stage;
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  renderActions: (card: BoardCard) => ReactNode;
};

export function BoardColumn({ stage, cards, onOpen, renderActions }: Props) {
  const { t } = useTranslation();
  const headingId = `column-${stage}`;
  const isOffer = stage === "offer";
  return (
    <KanbanColumn
      value={stage}
      render={<section />}
      aria-labelledby={headingId}
      className={cn(
        "noir-panel flex min-h-0 flex-col rounded-xl data-dragging:border-noir-amber",
        isOffer && "border-noir-teal/50",
      )}
    >
      <header className="flex items-center gap-2 px-4 py-3">
        <h2 id={headingId} className={cn("text-sm font-semibold", isOffer && "text-noir-teal")}>
          {t(`stage.${stage}`)}
        </h2>
        <span className="rounded bg-white/5 px-1.5 font-mono text-xs text-noir-muted">
          <span aria-hidden="true">{cards.length}</span>
          <span className="sr-only">{t("board.count", { count: cards.length })}</span>
        </span>
      </header>
      {cards.length === 0 ? (
        <p className="px-4 pb-4 text-sm text-noir-muted">{t("board.empty")}</p>
      ) : (
        <ul className="flex flex-col gap-3 overflow-y-auto px-3 pb-3">
          {cards.map((card) => (
            <KanbanItem key={card.id} value={card.id} render={<li />}>
              <JobCard card={card} onOpen={onOpen} actions={renderActions(card)} />
            </KanbanItem>
          ))}
        </ul>
      )}
    </KanbanColumn>
  );
}
```

`src/widgets/board/Board.tsx` (full replacement):
```tsx
import { GripVertical } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { STAGES } from "@/entities/job/model/stages";
import { JobCard } from "@/entities/job/ui/JobCard";
import { MoveMenu } from "@/features/move-card/ui/MoveMenu";
import { OutcomeDialog } from "@/features/move-card/ui/OutcomeDialog";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import { Kanban, KanbanBoard, KanbanItemHandle, KanbanOverlay } from "@/shared/ui/kanban";
import { BoardColumn } from "./BoardColumn";
import { buildAnnouncements } from "./model/announcements";
import { columnOf, type Columns } from "./model/kanban";
import { useKanbanColumns } from "./model/use-kanban-columns";

type Props = {
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  onMove: (id: string, stage: Stage, outcome: Outcome | null) => void;
};

export function Board({ cards, onOpen, onMove }: Props) {
  const { t } = useTranslation();
  const { columns, setColumns, begin, finish, restore } = useKanbanColumns(cards);
  const [pendingClose, setPendingClose] = useState<string | null>(null);
  const announcements = buildAnnouncements(t, cards, (id) => columnOf(columns, id));

  const handleDragEnd = () => {
    const drop = finish();
    if (!drop) return;
    // Closing needs an outcome; the card waits in Encerradas until one is picked.
    if (drop.kind === "close") setPendingClose(drop.id);
    else onMove(drop.id, drop.stage, null);
  };

  return (
    <>
      <Kanban
        value={columns}
        onValueChange={(next) => setColumns(next as Columns)}
        getItemValue={(card) => card.id}
        accessibility={{ announcements, screenReaderInstructions: { draggable: t("dnd.instructions") } }}
        onDragStart={({ active }) => begin(String(active.id))}
        onDragEnd={handleDragEnd}
        onDragCancel={() => {
          finish();
          restore();
        }}
      >
        <KanbanBoard className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto pb-2">
          {STAGES.map((stage) => (
            <BoardColumn
              key={stage}
              stage={stage}
              cards={columns[stage]}
              onOpen={onOpen}
              renderActions={(card) => (
                <>
                  <KanbanItemHandle
                    aria-label={t("card.drag", { title: card.title })}
                    className="grid size-7 place-items-center rounded-md text-noir-muted hover:bg-white/5 focus-visible:outline-2 focus-visible:outline-noir-amber"
                  >
                    <GripVertical aria-hidden="true" className="size-4" />
                  </KanbanItemHandle>
                  <MoveMenu card={card} onMove={(target, outcome) => onMove(card.id, target, outcome)} />
                </>
              )}
            />
          ))}
        </KanbanBoard>
        <KanbanOverlay>
          {({ value }) => {
            const card = cards.find((item) => item.id === value);
            return card ? <JobCard card={card} onOpen={() => {}} /> : null;
          }}
        </KanbanOverlay>
      </Kanban>
      <OutcomeDialog
        open={pendingClose !== null}
        onCancel={() => {
          setPendingClose(null);
          restore();
        }}
        onPick={(outcome) => {
          if (pendingClose) onMove(pendingClose, "closed", outcome);
          setPendingClose(null);
        }}
      />
    </>
  );
}
```
Then delete `src/widgets/board/model/drop.ts` and `src/widgets/board/model/drop.test.ts` (replaced by `resolveKanbanDrop`), and remove `@dnd-kit/core`-only imports left anywhere else (`DndContext`, `DragOverlay`, `PointerSensor`).

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ Board labels each drag handle`, `✓ Board keeps the move menu next to the handle`, `✓ useBoard hands out a fresh list after a failed move so views resync`, all earlier Board and BoardPage tests)

Manual: `pnpm tauri dev`: drag a card by its grip to Triagem; Tab to a grip, Space, arrow right twice, Space; drag to Encerradas then Cancelar (card returns); Esc during a drag (card returns).

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: Job details in Dice UI ResponsiveDialog

**Requirement:** REQ-005
**Files:**
- Modify: `src/entities/job/ui/JobDialog.tsx`
- Test: `src/entities/job/ui/JobDialog.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Append inside `describe("JobDialog", ...)` in `src/entities/job/ui/JobDialog.test.tsx`:
```tsx
  it("renders the dialog variant on desktop", () => {
    render(<JobDialog card={card} onOpenChange={() => {}} />);
    expect(screen.getByRole("dialog")).toHaveAttribute("data-variant", "dialog");
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/entities/job/ui/JobDialog.test.tsx`
Expected: FAIL with "Expected the element to have attribute: data-variant="dialog""

**Step 3: Write minimal implementation**

`src/entities/job/ui/JobDialog.tsx` (full replacement):
```tsx
import { Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { Button } from "@/shared/ui/button";
import {
  ResponsiveDialog,
  ResponsiveDialogClose,
  ResponsiveDialogContent,
  ResponsiveDialogDescription,
  ResponsiveDialogFooter,
  ResponsiveDialogHeader,
  ResponsiveDialogTitle,
} from "@/shared/ui/responsive-dialog";
import { hasNegotiatedSalary } from "../model/format";
import { SalaryLine } from "./SalaryLine";

type Props = { card: BoardCard | null; onOpenChange: (open: boolean) => void };

const ARTIFACTS = ["cv", "coverage", "interviewPrep"] as const;

export function JobDialog({ card, onOpenChange }: Props) {
  const { t } = useTranslation();
  return (
    <ResponsiveDialog open={card !== null} onOpenChange={onOpenChange}>
      {card && (
        <ResponsiveDialogContent showCloseButton={false} className="noir-panel max-w-2xl">
          <ResponsiveDialogHeader>
            <ResponsiveDialogTitle>{card.title}</ResponsiveDialogTitle>
            <ResponsiveDialogDescription>
              {[card.company, card.source, t(`target.${card.target}`), t(`workModel.${card.workModel}`)].join(" · ")}
            </ResponsiveDialogDescription>
          </ResponsiveDialogHeader>

          <section aria-labelledby="job-dialog-salary" className="flex flex-col gap-1">
            <h3 id="job-dialog-salary" className="text-xs font-semibold uppercase text-noir-muted">
              {t("dialog.salary")}
            </h3>
            {hasNegotiatedSalary(card.salary) ? (
              <SalaryLine salary={card.salary} />
            ) : (
              <p className="text-sm text-noir-muted">{t("dialog.noSalary")}</p>
            )}
          </section>

          <section aria-labelledby="job-dialog-artifacts" className="flex flex-col gap-1">
            <h3 id="job-dialog-artifacts" className="text-xs font-semibold uppercase text-noir-muted">
              {t("dialog.artifacts")}
            </h3>
            <ul className="text-sm">
              {ARTIFACTS.map((key) => (
                <li key={key}>
                  {t(`card.artifacts.${key}`)}:{" "}
                  {t(card.artifacts[key] ? "card.artifacts.done" : "card.artifacts.missing")}
                </li>
              ))}
            </ul>
          </section>

          <section aria-labelledby="job-dialog-timeline" className="flex flex-col gap-1">
            <h3 id="job-dialog-timeline" className="text-xs font-semibold uppercase text-noir-muted">
              {t("dialog.timeline")}
            </h3>
            <ol className="flex flex-col gap-1">
              {card.timeline.map((event) => (
                <li key={`${event.at}-${event.kind}`} className="flex gap-3 text-sm">
                  <time dateTime={event.at} className="font-mono text-noir-muted">
                    {event.at}
                  </time>
                  <span>{event.label}</span>
                </li>
              ))}
            </ol>
          </section>

          <ResponsiveDialogFooter className="items-center gap-3">
            <span id="job-dialog-cv-hint" className="text-xs text-noir-muted">
              {t("dialog.comingSoon")}
            </span>
            {/* Enabled once the AI CLI runner exists (later plan). */}
            <Button disabled aria-describedby="job-dialog-cv-hint">
              <Sparkles aria-hidden="true" />
              {t("dialog.generateCv")}
            </Button>
            <ResponsiveDialogClose render={<Button variant="ghost" />}>{t("dialog.close")}</ResponsiveDialogClose>
          </ResponsiveDialogFooter>
        </ResponsiveDialogContent>
      )}
    </ResponsiveDialog>
  );
}
```
If `data-variant` lands on an inner element rather than the `role="dialog"` node, assert on `screen.getByRole("dialog").closest('[data-variant]')` instead and keep the test name.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ JobDialog renders the dialog variant on desktop`, the three existing JobDialog tests, `BoardPage opens the details of a card`)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: Inline Field filters (Select, Input, Checkbox)

**Requirement:** REQ-008
**Files:**
- Modify: `src/features/filter-board/model/filter.ts`
- Modify: `src/features/filter-board/ui/Toolbar.tsx`
- Modify: `src/shared/locales/pt-BR.json`
- Modify: `src/shared/locales/en.json`
- Test: `src/features/filter-board/model/filter.test.ts`
- Test: `src/features/filter-board/ui/Toolbar.test.tsx`
- Test: `src/pages/board/BoardPage.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: business-logic-reviewer, type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/features/filter-board/model/filter.test.ts` (full replacement):
```ts
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { EMPTY_FILTERS, clampScore, filterCards, matchesText } from "./filter";

const v = makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde", source: "Gmail", target: "ai_llm", score: 91, unread: true });
const n = makeCard({ id: "n", title: "Engenheiro de Dados", company: "Nébula Data", source: "Lever", target: "data", score: 79 });
const o = makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay", source: "Gupy", target: "tech_lead", score: 82 });
const ids = (cards: { id: string }[]) => cards.map((c) => c.id);

describe("filterCards", () => {
  it("keeps every card with empty filters", () => {
    expect(ids(filterCards([v, n, o], EMPTY_FILTERS))).toEqual(["v", "n", "o"]);
  });

  it("filters by source and target", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Gupy"], targets: ["tech_lead"] }))).toEqual(["o"]);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Lever", "Gmail"] }))).toEqual(["v", "n"]);
  });

  it("drops cards below the minimum score", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, minScore: 80 }))).toEqual(["v", "o"]);
  });

  it("keeps only unread cards when asked", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, unreadOnly: true }))).toEqual(["v"]);
  });
});

describe("matchesText", () => {
  it("matches titles and companies without accents", () => {
    expect(matchesText("Vértice Saúde", "vertice")).toBe(true);
    expect(matchesText("Engenheiro de Dados Sênior", "SENIOR")).toBe(true);
    expect(matchesText("Orbita Pay", "vertice")).toBe(false);
  });
});

describe("clampScore", () => {
  it("keeps the score between 0 and 100", () => {
    expect(clampScore("85")).toBe(85);
    expect(clampScore("150")).toBe(100);
    expect(clampScore("-5")).toBe(0);
    expect(clampScore("")).toBe(0);
    expect(clampScore("abc")).toBe(0);
  });
});
```

`src/features/filter-board/ui/Toolbar.test.tsx` (full replacement):
```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EMPTY_FILTERS } from "../model/filter";
import { Toolbar } from "./Toolbar";

const sources = ["Gupy", "Lever"];

describe("Toolbar", () => {
  it("toggles a source filter", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />);
    await userEvent.click(screen.getByRole("combobox", { name: "Fonte" }));
    await userEvent.click(await screen.findByRole("option", { name: "Gupy" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, sources: ["Gupy"] });
  });

  it("toggles a target filter", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />);
    await userEvent.click(screen.getByRole("combobox", { name: "Alvo" }));
    await userEvent.click(await screen.findByRole("option", { name: "Dados" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, targets: ["data"] });
  });

  it("clamps the minimum score", () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />);
    fireEvent.change(screen.getByRole("spinbutton", { name: "Score mín." }), { target: { value: "150" } });
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, minScore: 100 });
  });

  it("filters unread only", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />);
    await userEvent.click(screen.getByRole("checkbox", { name: "Somente não lidas" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, unreadOnly: true });
  });

  it("summarizes the selection in the trigger", () => {
    render(<Toolbar filters={{ ...EMPTY_FILTERS, sources: ["Gupy", "Lever"] }} sources={sources} onChange={() => {}} />);
    expect(screen.getByRole("combobox", { name: "Fonte" })).toHaveTextContent("2 selecionados");
  });
});
```

`src/pages/board/BoardPage.test.tsx`: replace the test `filters the board from the search box` with:
```tsx
  it("filters the board to unread cards", async () => {
    const list = [makeCard({ id: "u", title: "Unread Job", unread: true }), makeCard({ id: "r", title: "Read Job" })];
    render(<BoardPage api={fakeApi(list)} now={now} />);
    await screen.findByRole("heading", { name: /Read Job/ });
    await userEvent.click(screen.getByRole("checkbox", { name: "Somente não lidas" }));
    expect(screen.getByRole("heading", { name: /Unread Job/ })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: /^Read Job/ })).not.toBeInTheDocument();
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/features/filter-board src/pages/board/BoardPage.test.tsx`
Expected: FAIL with "clampScore is not a function" and "Unable to find an accessible element with the role "combobox" and name "Fonte""

**Step 3: Write minimal implementation**

`src/features/filter-board/model/filter.ts` (full replacement):
```ts
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Target } from "@/shared/bindings/Target";

export type BoardFilters = {
  sources: string[];
  targets: Target[];
  minScore: number;
  unreadOnly: boolean;
};

export const EMPTY_FILTERS: BoardFilters = { sources: [], targets: [], minScore: 0, unreadOnly: false };

/** Lowercase without diacritics, so "vertice" finds "Vértice". */
export const fold = (value: string) =>
  value.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase("pt-BR");

/** Accent-insensitive substring match, shared with the command palette. */
export function matchesText(text: string, query: string): boolean {
  return fold(text).includes(fold(query.trim()));
}

/** The score field is free text; anything outside 0-100 or not a number is coerced. */
export function clampScore(raw: string): number {
  const value = Number.parseInt(raw, 10);
  if (Number.isNaN(value)) return 0;
  return Math.min(100, Math.max(0, value));
}

export function filterCards(cards: BoardCard[], filters: BoardFilters): BoardCard[] {
  return cards.filter(
    (card) =>
      (filters.sources.length === 0 || filters.sources.includes(card.source)) &&
      (filters.targets.length === 0 || filters.targets.includes(card.target)) &&
      card.score >= filters.minScore &&
      (!filters.unreadOnly || card.unread),
  );
}
```

`src/features/filter-board/ui/Toolbar.tsx` (full replacement):
```tsx
import { Mail, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TARGETS } from "@/entities/job/model/stages";
import type { Target } from "@/shared/bindings/Target";
import { Button } from "@/shared/ui/button";
import { Checkbox } from "@/shared/ui/checkbox";
import { Field, FieldLabel, FieldLegend, FieldSet } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/shared/ui/select";
import { clampScore, type BoardFilters } from "../model/filter";

type Props = {
  filters: BoardFilters;
  sources: string[];
  onChange: (filters: BoardFilters) => void;
  /** Opens the command palette; the button is hidden when absent. */
  onSearch?: () => void;
};

export function Toolbar({ filters, sources, onChange, onSearch }: Props) {
  const { t } = useTranslation();
  const summary = (count: number) => (count === 0 ? t("filters.all") : t("filters.selected", { count }));

  return (
    <FieldSet className="flex flex-1 flex-row flex-wrap items-end gap-4">
      <FieldLegend className="sr-only">{t("filters.label")}</FieldLegend>

      {onSearch && (
        <Button variant="outline" className="h-9 w-64 justify-between text-noir-muted" onClick={onSearch}>
          <span className="flex items-center gap-2">
            <Search aria-hidden="true" />
            {t("command.open")}
          </span>
          <kbd className="font-mono text-xs">{t("command.shortcut")}</kbd>
        </Button>
      )}

      <Field className="w-48">
        <FieldLabel htmlFor="filter-source">{t("filters.source")}</FieldLabel>
        <Select
          multiple
          value={filters.sources}
          onValueChange={(value) => onChange({ ...filters, sources: value as string[] })}
        >
          <SelectTrigger id="filter-source" className="h-9 w-full">
            <SelectValue>{(value: string[]) => summary(value.length)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {sources.map((source) => (
              <SelectItem key={source} value={source}>
                {source}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>

      <Field className="w-48">
        <FieldLabel htmlFor="filter-target">{t("filters.target")}</FieldLabel>
        <Select
          multiple
          value={filters.targets}
          onValueChange={(value) => onChange({ ...filters, targets: value as Target[] })}
        >
          <SelectTrigger id="filter-target" className="h-9 w-full">
            <SelectValue>{(value: Target[]) => summary(value.length)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {TARGETS.map((target) => (
              <SelectItem key={target} value={target}>
                {t(`target.${target}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>

      <Field className="w-28">
        <FieldLabel htmlFor="filter-score">{t("filters.minScore")}</FieldLabel>
        <Input
          id="filter-score"
          type="number"
          inputMode="numeric"
          min={0}
          max={100}
          step={5}
          value={filters.minScore}
          onChange={(event) => onChange({ ...filters, minScore: clampScore(event.target.value) })}
          className="h-9 font-mono"
        />
      </Field>

      <Field orientation="horizontal" className="h-9 w-auto items-center">
        <Checkbox
          id="filter-unread"
          checked={filters.unreadOnly}
          onCheckedChange={(checked) => onChange({ ...filters, unreadOnly: checked === true })}
        />
        <FieldLabel htmlFor="filter-unread">{t("filters.unreadOnly")}</FieldLabel>
      </Field>

      {/* Gmail sync arrives with the AI CLI runner; disabled until then. */}
      <Button variant="outline" className="ml-auto h-9" disabled title={t("dialog.comingSoon")}>
        <Mail aria-hidden="true" />
        {t("filters.syncGmail")}
      </Button>
    </FieldSet>
  );
}
```
If Base UI's `Select` rejects `multiple` with a function child in `SelectValue`, keep `multiple` and render the summary with a plain `<span>` inside the trigger instead of `SelectValue`; the tests and the check names stay.

The free-text search leaves the board (Task 8 adds the palette), so drop its string:
```bash
for f in src/shared/locales/pt-BR.json src/shared/locales/en.json; do
  tmp=$(mktemp); jq 'del(.filters.search)' "$f" > "$tmp" && [ -s "$tmp" ] && mv "$tmp" "$f"
done
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ Toolbar clamps the minimum score`, `✓ Toolbar filters unread only`, the other Toolbar and filter tests, `✓ BoardPage filters the board to unread cards`, locale parity)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 8: Ctrl/Cmd+K command palette that opens a job

**Requirement:** REQ-007
**Files:**
- Create: `src/features/search-jobs/ui/JobCommand.tsx`
- Create: `src/shared/lib/hooks/use-hotkey.ts`
- Modify: `src/pages/board/BoardPage.tsx`
- Test: `src/features/search-jobs/ui/JobCommand.test.tsx`
- Test: `src/shared/lib/hooks/use-hotkey.test.ts`
- Test: `src/pages/board/BoardPage.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/shared/lib/hooks/use-hotkey.test.ts`:
```ts
import { fireEvent } from "@testing-library/react";
import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useHotkey } from "./use-hotkey";

describe("useHotkey", () => {
  it("fires on Ctrl or Cmd plus the key", () => {
    const handler = vi.fn();
    renderHook(() => useHotkey("k", handler));
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    fireEvent.keyDown(window, { key: "K", metaKey: true });
    expect(handler).toHaveBeenCalledTimes(2);
  });

  it("ignores the bare key", () => {
    const handler = vi.fn();
    renderHook(() => useHotkey("k", handler));
    fireEvent.keyDown(window, { key: "k" });
    expect(handler).not.toHaveBeenCalled();
  });
});
```

`src/features/search-jobs/ui/JobCommand.test.tsx`:
```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { JobCommand } from "./JobCommand";

const cards = [
  makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde" }),
  makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay" }),
];

describe("JobCommand", () => {
  it("lists every job when empty", () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    expect(screen.getAllByRole("option")).toHaveLength(2);
  });

  it("finds a job without accents", async () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    await userEvent.type(screen.getByRole("combobox"), "vertice");
    expect(screen.getAllByRole("option")).toHaveLength(1);
    expect(screen.getByRole("option")).toHaveTextContent("Lead AI Architect");
  });

  it("shows the empty state", async () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    await userEvent.type(screen.getByRole("combobox"), "zzz");
    expect(screen.getByText("Nenhuma vaga encontrada.")).toBeInTheDocument();
  });

  it("picks a job and closes", async () => {
    const onPick = vi.fn();
    const onOpenChange = vi.fn();
    render(<JobCommand cards={cards} open onOpenChange={onOpenChange} onPick={onPick} />);
    await userEvent.click(screen.getByRole("option", { name: /Tech Lead/ }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onPick).toHaveBeenCalledWith(cards[1]);
  });
});
```

Append inside `describe("BoardPage", ...)` in `src/pages/board/BoardPage.test.tsx`:
```tsx
  it("opens a job from the command palette", async () => {
    render(<BoardPage api={fakeApi(cards)} now={now} />);
    await screen.findByRole("heading", { name: /Lead AI Architect/ });
    await userEvent.keyboard("{Control>}k{/Control}");
    await userEvent.type(await screen.findByRole("combobox"), "vertice");
    await userEvent.click(screen.getByRole("option", { name: /Lead AI Architect/ }));
    const dialog = await screen.findByRole("dialog", { name: "Lead AI Architect" });
    expect(within(dialog).getByRole("button", { name: "Gerar CV customizado" })).toBeDisabled();
  });
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/features/search-jobs src/shared/lib/hooks src/pages/board/BoardPage.test.tsx`
Expected: FAIL with "Failed to resolve import "./JobCommand"" and "Failed to resolve import "./use-hotkey""

**Step 3: Write minimal implementation**

`src/shared/lib/hooks/use-hotkey.ts`:
```ts
import { useEffect } from "react";

/** Ctrl+key on Linux/Windows, Cmd+key on macOS. `handler` should be stable. */
export function useHotkey(key: string, handler: () => void): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() !== key || !(event.ctrlKey || event.metaKey)) return;
      event.preventDefault();
      handler();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [key, handler]);
}
```

`src/features/search-jobs/ui/JobCommand.tsx`:
```tsx
import { useTranslation } from "react-i18next";
import { matchesText } from "@/features/filter-board/model/filter";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/shared/ui/command";

type Props = {
  cards: BoardCard[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onPick: (card: BoardCard) => void;
};

export function JobCommand({ cards, open, onOpenChange, onPick }: Props) {
  const { t } = useTranslation();
  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title={t("command.title")}
      description={t("command.description")}
    >
      {/* cmdk's fuzzy score ignores accents poorly; match like the board filters do. */}
      <Command label={t("command.title")} filter={(value, search) => (matchesText(value, search) ? 1 : 0)}>
        <CommandInput placeholder={t("command.placeholder")} />
        <CommandList>
          <CommandEmpty>{t("command.empty")}</CommandEmpty>
          <CommandGroup heading={t("command.group")}>
            {cards.map((card) => (
              <CommandItem
                key={card.id}
                // The id keeps two jobs with the same title and company distinct.
                value={`${card.title} ${card.company} ${card.id}`}
                onSelect={() => {
                  onOpenChange(false);
                  onPick(card);
                }}
              >
                <span className="flex-1 truncate">{card.title}</span>
                <span className="truncate text-xs text-noir-muted">{card.company}</span>
                <span className="font-mono text-xs">{card.score}</span>
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
```

`src/pages/board/BoardPage.tsx` (full replacement):
```tsx
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { JobDialog } from "@/entities/job/ui/JobDialog";
import { EMPTY_FILTERS, filterCards, type BoardFilters } from "@/features/filter-board/model/filter";
import { Toolbar } from "@/features/filter-board/ui/Toolbar";
import { JobCommand } from "@/features/search-jobs/ui/JobCommand";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { useHotkey } from "@/shared/lib/hooks/use-hotkey";
import { Board } from "@/widgets/board/Board";
import { MetricsStrip } from "@/widgets/board/MetricsStrip";
import { computeMetrics } from "@/widgets/board/model/metrics";

type Props = { api?: BoardApi; now?: Date };

export function BoardPage({ api = boardApi, now }: Props) {
  const { t } = useTranslation();
  const { cards, status, moveError, move } = useBoard(api);
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_FILTERS);
  const [openId, setOpenId] = useState<string | null>(null);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const visible = useMemo(() => filterCards(cards, filters), [cards, filters]);
  // Metrics follow the filters, so a source or target can be judged on its own.
  const metrics = useMemo(() => computeMetrics(visible, now ?? new Date()), [visible, now]);
  const sources = useMemo(() => [...new Set(cards.map((card) => card.source))].sort(), [cards]);
  const openCard = cards.find((card) => card.id === openId) ?? null;
  const openPalette = useCallback(() => setPaletteOpen(true), []);
  useHotkey("k", openPalette);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <div className="flex flex-wrap items-end gap-4">
        <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
        <Toolbar filters={filters} sources={sources} onChange={setFilters} onSearch={openPalette} />
      </div>
      <MetricsStrip metrics={metrics} />
      {moveError && (
        <p role="alert" className="text-sm text-noir-red">
          {t("board.moveError")}
        </p>
      )}
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && (
        <Board
          cards={visible}
          onOpen={(card) => setOpenId(card.id)}
          onMove={(id, stage, outcome) => void move(id, stage, outcome)}
        />
      )}
      {/* The palette searches every job, not only the filtered ones. */}
      <JobCommand cards={cards} open={paletteOpen} onOpenChange={setPaletteOpen} onPick={(card) => setOpenId(card.id)} />
      <JobDialog card={openCard} onOpenChange={(open) => !open && setOpenId(null)} />
    </div>
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ JobCommand finds a job without accents`, `✓ BoardPage opens a job from the command palette`, `✓ useHotkey fires on Ctrl or Cmd plus the key` and the rest)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 9: shadcn Sidebar collapsed to icons, remembered across launches

**Requirement:** REQ-009
**Files:**
- Create: `src/shared/lib/sidebar-state.ts`
- Modify: `src/widgets/app-shell/Sidebar.tsx`
- Modify: `src/widgets/app-shell/AppShell.tsx`
- Modify: `src/shared/styles/theme.css`
- Test: `src/shared/lib/sidebar-state.test.ts`
- Test: `src/widgets/app-shell/AppShell.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/shared/lib/sidebar-state.test.ts`:
```ts
import { afterEach, describe, expect, it, vi } from "vitest";
import { SIDEBAR_KEY, readSidebarOpen, writeSidebarOpen } from "./sidebar-state";

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("sidebar state", () => {
  it("starts collapsed when nothing was stored", () => {
    expect(readSidebarOpen()).toBe(false);
  });

  it("round-trips the stored choice", () => {
    writeSidebarOpen(true);
    expect(localStorage.getItem(SIDEBAR_KEY)).toBe("true");
    expect(readSidebarOpen()).toBe(true);
  });

  it("survives unavailable storage", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("denied");
    });
    expect(readSidebarOpen()).toBe(false);
    expect(() => writeSidebarOpen(true)).not.toThrow();
  });
});
```

`src/widgets/app-shell/AppShell.test.tsx` (full replacement):
```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SIDEBAR_KEY } from "@/shared/lib/sidebar-state";
import { AppShell } from "./AppShell";

const controls = () => ({
  minimize: vi.fn(async () => {}),
  toggleMaximize: vi.fn(async () => {}),
  close: vi.fn(async () => {}),
});
const sidebarState = (container: HTMLElement) =>
  container.querySelector('[data-slot="sidebar"]')?.getAttribute("data-state");

afterEach(() => localStorage.clear());

describe("AppShell", () => {
  it("switches the page from the sidebar", async () => {
    render(<AppShell controls={controls()} renderPage={(page) => <h1>{page}</h1>} />);
    expect(screen.getByRole("heading", { name: "board" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Meu perfil/ }));
    expect(screen.getByRole("heading", { name: "profile" })).toBeInTheDocument();
  });

  it("marks the active item", async () => {
    render(<AppShell controls={controls()} renderPage={() => null} />);
    await userEvent.click(screen.getByRole("button", { name: /Configuração/ }));
    expect(screen.getByRole("button", { name: /Configuração/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("button", { name: /^Board/ })).not.toHaveAttribute("aria-current");
  });

  it("drives the window controls", async () => {
    const window = controls();
    render(<AppShell controls={window} renderPage={() => null} />);
    await userEvent.click(screen.getByRole("button", { name: "Minimizar" }));
    await userEvent.click(screen.getByRole("button", { name: "Maximizar" }));
    await userEvent.click(screen.getByRole("button", { name: "Fechar" }));
    expect(window.minimize).toHaveBeenCalledOnce();
    expect(window.toggleMaximize).toHaveBeenCalledOnce();
    expect(window.close).toHaveBeenCalledOnce();
  });

  it("starts collapsed and remembers the choice", async () => {
    const { container } = render(<AppShell controls={controls()} renderPage={() => null} />);
    expect(sidebarState(container)).toBe("collapsed");
    await userEvent.click(screen.getByRole("button", { name: "Alternar barra lateral" }));
    expect(sidebarState(container)).toBe("expanded");
    expect(localStorage.getItem(SIDEBAR_KEY)).toBe("true");
  });

  it("reopens expanded when that was the last choice", () => {
    localStorage.setItem(SIDEBAR_KEY, "true");
    const { container } = render(<AppShell controls={controls()} renderPage={() => null} />);
    expect(sidebarState(container)).toBe("expanded");
  });

  it("toggles with Ctrl+B", async () => {
    const { container } = render(<AppShell controls={controls()} renderPage={() => null} />);
    await userEvent.keyboard("{Control>}b{/Control}");
    expect(sidebarState(container)).toBe("expanded");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/shared/lib/sidebar-state.test.ts src/widgets/app-shell`
Expected: FAIL with "Failed to resolve import "./sidebar-state"" and "Failed to resolve import "@/shared/lib/sidebar-state""

**Step 3: Write minimal implementation**

`src/shared/lib/sidebar-state.ts`:
```ts
export const SIDEBAR_KEY = "job-radar.sidebar-open";

/** Collapsed (icons only) unless the user expanded it last time. */
export function readSidebarOpen(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_KEY) === "true";
  } catch {
    return false;
  }
}

export function writeSidebarOpen(open: boolean): void {
  try {
    localStorage.setItem(SIDEBAR_KEY, String(open));
  } catch {
    // Storage unavailable (private mode, blocked): the in-memory state still works.
  }
}
```

`src/widgets/app-shell/Sidebar.tsx` (full replacement; the export is renamed to avoid clashing with shadcn's `Sidebar`):
```tsx
import {
  FileText,
  LayoutGrid,
  NotebookPen,
  PanelLeft,
  Settings,
  UserRound,
  type LucideIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { PageId } from "@/shared/config/navigation";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/shared/ui/sidebar";

type NavItem = { id: PageId; Icon: LucideIcon; pending?: number };

// Pending counts and crawler status are mocks until the store backs them.
const NAV: NavItem[] = [
  { id: "board", Icon: LayoutGrid },
  { id: "profile", Icon: UserRound },
  { id: "resumes", Icon: FileText, pending: 3 },
  { id: "prep", Icon: NotebookPen, pending: 2 },
  { id: "settings", Icon: Settings },
];
const CRAWLER_STATUS = { time: "08:00", total: 142, fresh: 9 };

type Props = { active: PageId; onNavigate: (page: PageId) => void };

export function AppSidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  const { state, toggleSidebar } = useSidebar();
  return (
    // The custom title bar takes the top 2.5rem, so the fixed sidebar starts below it.
    <Sidebar collapsible="icon" className="!top-10 !h-[calc(100svh-2.5rem)]">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              onClick={toggleSidebar}
              tooltip={t("sidebar.toggle")}
              aria-expanded={state === "expanded"}
            >
              <PanelLeft aria-hidden="true" />
              <span>{t("sidebar.toggle")}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <nav aria-label={t("nav.label")}>
              <SidebarMenu>
                {NAV.map(({ id, Icon, pending }) => (
                  <SidebarMenuItem key={id}>
                    <SidebarMenuButton
                      isActive={active === id}
                      aria-current={active === id ? "page" : undefined}
                      tooltip={t(`nav.${id}`)}
                      onClick={() => onNavigate(id)}
                    >
                      <Icon aria-hidden="true" />
                      <span>{t(`nav.${id}`)}</span>
                    </SidebarMenuButton>
                    {pending !== undefined && (
                      <SidebarMenuBadge className="font-mono text-noir-amber">
                        <span aria-hidden="true">{pending}</span>
                        <span className="sr-only">{t("nav.pending", { count: pending })}</span>
                      </SidebarMenuBadge>
                    )}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </nav>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="gap-1 text-xs text-noir-muted">
        <p className="flex items-center gap-2 px-2 text-noir-teal">
          <span aria-hidden="true" className="size-2 shrink-0 rounded-full bg-noir-teal" />
          <span className="group-data-[collapsible=icon]:sr-only">{t("crawler.active")}</span>
        </p>
        <p className="px-2 group-data-[collapsible=icon]:hidden">{t("crawler.lastRun", CRAWLER_STATUS)}</p>
        <p className="mt-2 px-2 font-serif italic group-data-[collapsible=icon]:hidden">{t("app.signature")} •</p>
      </SidebarFooter>
    </Sidebar>
  );
}
```

`src/widgets/app-shell/AppShell.tsx` (full replacement):
```tsx
import { useState, type ReactNode } from "react";
import type { PageId } from "@/shared/config/navigation";
import { readSidebarOpen, writeSidebarOpen } from "@/shared/lib/sidebar-state";
import { SidebarInset, SidebarProvider } from "@/shared/ui/sidebar";
import { AppSidebar } from "./Sidebar";
import { TitleBar, type WindowControls } from "./TitleBar";

type Props = {
  renderPage: (page: PageId) => ReactNode;
  controls?: WindowControls;
};

export function AppShell({ renderPage, controls }: Props) {
  const [page, setPage] = useState<PageId>("board");
  const [sidebarOpen, setSidebarOpen] = useState(readSidebarOpen);
  // Controlled so the choice lives in localStorage; shadcn's cookie targets SSR.
  const handleSidebar = (open: boolean) => {
    setSidebarOpen(open);
    writeSidebarOpen(open);
  };
  return (
    <div className="noir-backdrop flex h-full flex-col">
      <TitleBar controls={controls} />
      <SidebarProvider open={sidebarOpen} onOpenChange={handleSidebar} className="min-h-0 flex-1">
        <AppSidebar active={page} onNavigate={setPage} />
        <SidebarInset className="min-w-0 bg-transparent">{renderPage(page)}</SidebarInset>
      </SidebarProvider>
    </div>
  );
}
```

Append to `src/shared/styles/theme.css` (sidebar tokens on the Noir palette; replace any values the shadcn CLI wrote for these names):
```css
:root {
  --sidebar: #0e0e10;
  --sidebar-foreground: #ededed;
  --sidebar-primary: #f5a524;
  --sidebar-primary-foreground: #0b0b0b;
  --sidebar-accent: #1c1c1f;
  --sidebar-accent-foreground: #ededed;
  --sidebar-border: #26262b;
  --sidebar-ring: #f5a524;
}

@theme inline {
  --color-sidebar: var(--sidebar);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-ring: var(--sidebar-ring);
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ AppShell starts collapsed and remembers the choice`, `toggles with Ctrl+B`, the 3 sidebar state tests, `✓ App shows a placeholder for pages not built yet`)

Manual: `pnpm tauri dev`: sidebar starts as icons below the title bar; hover shows tooltips; Ctrl+B expands; restart keeps the last state.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 10: Accessibility gate with the palette open, and final gates

**Requirement:** REQ-010
**Files:**
- Test: `src/pages/board/BoardPage.a11y.test.tsx`

**Team:**
- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

Append inside `describe("BoardPage", ...)` in `src/pages/board/BoardPage.a11y.test.tsx`:
```tsx
  it("has no axe violations with the palette open", async () => {
    render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { level: 2, name: "Caixa de entrada" });
    await userEvent.keyboard("{Control>}k{/Control}");
    await screen.findByRole("combobox");
    await expectNoAxeViolations(document.body);
  });
```
(Add `import userEvent from "@testing-library/user-event";` at the top.)

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/pages/board/BoardPage.a11y.test.tsx`
Expected: FAIL with the axe report listing at least one violation id (for example a portal or label issue) on the first run; if it passes on the first run, record that in STATE.md and keep the test as a regression guard.

**Step 3: Write minimal implementation**

Fix every reported violation in the component that owns the node (a missing label on a Base UI control, a duplicate id from the overlay card, a region without a name), never by disabling an axe rule. Typical fixes:
- `aria-label` or `FieldLabel htmlFor` on a control without an accessible name;
- pass `label` to `Command` when the input has no name;
- give the overlay `JobCard` a distinct title id (`job-overlay-<id>-title`) if axe reports `duplicate-id-aria`.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm vitest run --coverage && pnpm typecheck && pnpm lint && pnpm build && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS (`✓ BoardPage has no axe violations`, `✓ BoardPage has no axe violations with the palette open`, coverage at or above 80% on every metric, typecheck, lint, build and Rust tests green)

Manual (Niri, `pnpm tauri dev`): pointer and keyboard drag, drop on Encerradas then Cancelar, Ctrl+K then Enter on a result, Ctrl+B, sidebar state after restart, Status colors on the Noir background.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
