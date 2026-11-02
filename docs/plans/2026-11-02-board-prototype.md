# Plan: Board Prototype (setup + board screen with mocks)

> **Spec:** docs/specs/board-prototype/SPEC.md

**Goal:** Scaffold the Tauri 2 + React 19 app and ship the Terminal Noir board fed by Rust mock data over IPC.
**Architecture:** Rust owns the domain (`domain.rs`), the persistence boundary (`Store` trait, `MockStore` for now) and two Tauri commands; ts-rs exports the domain types to `src/shared/bindings`. The React side follows Feature-Sliced Design (`app`, `pages`, `widgets`, `features`, `entities`, `shared`), uses shadcn/ui on Tailwind 4 with Noir tokens, i18next for pt-BR/en and @dnd-kit/core for drag-and-drop.
**Tech Stack:** Tauri 2.12, Rust 1.96 (edition 2024), ts-rs 12, thiserror 2, React 19.3, TypeScript 6, Vite 8, Tailwind 4.3, shadcn 4.23 (radix-nova), i18next 26, @dnd-kit/core 6.3, Vitest 5 + Testing Library + jsdom, axe-core.
**Total Tasks:** 14
**Estimated Complexity:** large
**Lessons applied:** none

Conventions for every task:

- Commands run from the repo root (`~/dev/projects/job-radar-desktop`).
- Use `pnpm <script>` or `./node_modules/.bin/<tool>`, never a bare `tsc`/`eslint` (dead mise shims).
- Code, comments and identifiers in English; UI strings only through i18n keys (from Task 6 on).

Batches (dependency order): B1 = Tasks 1-2 (scaffold + spike), B2 = Tasks 3-5 (Rust), B3 = Tasks 6-7 (i18n + data hook), B4 = Tasks 8-10 (shell, card, board), B5 = Tasks 11-13 (filters, moves, dialog), B6 = Task 14 (a11y).

---

## Checks

**Run web**: `pnpm vitest run --reporter=verbose`
**Run rust**: `cargo test --manifest-path src-tauri/Cargo.toml`

- C1 (REQ-001): the app builds and type-checks. Proof: `pnpm build`
- C2 (REQ-001): the code lints clean. Proof: `pnpm lint`
- C3 (REQ-002): the visual mode defaults to glass and honors the env fallback. Proof: run web | ✓ resolveVisualMode defaults to glass
- C4 (REQ-003): enums cross IPC as snake_case. Proof: run rust | domain::tests::stage_serializes_as_snake_case ok
- C5 (REQ-003): the TypeScript bindings are generated from Rust. Proof: `[ -f src/shared/bindings/BoardCard.ts ]`
- C6 (REQ-004): `list_board` returns the seeded cards. Proof: run rust | commands::tests::list_board_returns_the_seeded_cards ok
- C7 (REQ-005): moving to closed requires an outcome. Proof: run rust | store::tests::moving_to_closed_requires_an_outcome ok
- C8 (REQ-005): an oversized id is rejected. Proof: run rust | commands::tests::move_card_rejects_an_oversized_id ok
- C9 (REQ-006): non-board pages show a placeholder. Proof: run web | ✓ App shows a placeholder for pages not built yet
- C10 (REQ-006): the title bar drives the window. Proof: run web | ✓ AppShell drives the window controls
- C11 (REQ-007): the six columns render in order. Proof: run web | ✓ Board renders the six columns in board order
- C12 (REQ-008): only the negotiated regime is shown. Proof: run web | ✓ JobCard shows only PJ when CLT was not negotiated
- C13 (REQ-009): a closed card shows its outcome. Proof: run web | ✓ JobCard shows the outcome of a closed card
- C14 (REQ-010): search ignores accents. Proof: run web | ✓ filterCards matches company names without accents
- C15 (REQ-010): metrics follow the cards. Proof: run web | ✓ computeMetrics computes the response rate
- C16 (REQ-011): the menu closes a card with an outcome. Proof: run web | ✓ Board closes a card with the chosen outcome
- C17 (REQ-012): CV generation stays disabled. Proof: run web | ✓ JobDialog keeps CV generation disabled until it exists
- C18 (REQ-013): locales share the same keys. Proof: run web | ✓ locales have the same keys in pt-BR and en
- C19 (REQ-014): the board page passes axe. Proof: run web | ✓ BoardPage has no axe violations

---

### Task 1: Scaffold Tauri 2 + React + toolchain

**Requirement:** REQ-001
**Files:**

- Create: `package.json`
- Create: `index.html`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `eslint.config.js`
- Create: `components.json`
- Create: `src/main.tsx`
- Create: `src/app/App.tsx`
- Create: `src/shared/lib/utils.ts`
- Create: `src/shared/styles/theme.css`
- Create: `src/test/setup.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/main.rs`
- Test: `src/app/App.test.tsx`

**Team:**

- Track: infra
- Implementer: infra-engineer (skills: bash-linux)
- Reviewers: security-reviewer, consequences-reviewer

**Step 1: Write the failing test**

```tsx
// src/app/App.test.tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the app name", () => {
    render(<App />);
    expect(screen.getByText("job-radar")).toBeInTheDocument();
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/app/App.test.tsx`
Expected: FAIL with "ERR_PNPM_NO_IMPORTER_MANIFEST_FOUND" (no package.json yet)

**Step 3: Write minimal implementation**

Scaffold next to `docs/` (verified non-interactive on 2026-11-02), then drop the demo files:

```bash
pnpm create tauri-app .scaffold -m pnpm -t react-ts -y
rsync -a --ignore-existing .scaffold/ ./ && rm -rf .scaffold
rm -rf README.md src/App.tsx src/App.css src/assets public/vite.svg public/tauri.svg .vscode
jq '.name = "job-radar-desktop"
  | .scripts = {dev: "vite", build: "tsc --noEmit && vite build", preview: "vite preview",
                typecheck: "tsc --noEmit", lint: "eslint .", test: "vitest run", tauri: "tauri"}' \
  package.json > package.json.tmp && [ -s package.json.tmp ] && mv package.json.tmp package.json
pnpm remove @tauri-apps/plugin-opener
pnpm add react@19.3.1 react-dom@19.3.1 @tauri-apps/api@2.12.4 i18next@26.7.1 react-i18next@17.1.11 \
  @dnd-kit/core@6.3.1 @dnd-kit/utilities@3.2.2 lucide-react@1.66.0 clsx tailwind-merge \
  class-variance-authority @fontsource-variable/inter@5.3.0 @fontsource/jetbrains-mono@5.3.0
pnpm add -D @tauri-apps/cli@2.12.4 typescript@6.0.3 tailwindcss@4.3.3 @tailwindcss/vite@4.3.3 \
  vitest@5.0.5 jsdom@30.1.3 @testing-library/react@16.3.3 @testing-library/user-event@14.6.8 \
  @testing-library/jest-dom@7.0.1 axe-core @types/node eslint@10.14.0 @eslint/js \
  typescript-eslint@8.72.1 eslint-plugin-react-hooks globals
pnpm audit --prod
```

`index.html`:

```html
<!doctype html>
<html lang="pt-BR">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>job-radar</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

`vite.config.ts`:

```ts
/// <reference types="vitest/config" />
import { fileURLToPath, URL } from "node:url";
import process from "node:process";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) } },
  // Tauri expects a fixed port and its own Rust errors on screen.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  test: { environment: "jsdom", setupFiles: ["./src/test/setup.ts"], css: false },
});
```

`eslint.config.js`:

```js
import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "src/shared/bindings"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
    },
  },
);
```

`components.json`:

```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "radix-nova",
  "rsc": false,
  "tsx": true,
  "tailwind": {
    "config": "",
    "css": "src/shared/styles/theme.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "iconLibrary": "lucide",
  "aliases": {
    "components": "@/shared/ui",
    "ui": "@/shared/ui",
    "utils": "@/shared/lib/utils",
    "lib": "@/shared/lib",
    "hooks": "@/shared/lib/hooks"
  }
}
```

`src/shared/lib/utils.ts`:

```ts
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
```

`src/shared/styles/theme.css`:

```css
@import "tailwindcss";
```

`src/test/setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

afterEach(() => cleanup());

// Radix primitives call browser APIs that jsdom does not implement.
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => {};
Element.prototype.scrollIntoView ??= () => {};
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;
```

`src/app/App.tsx`:

```tsx
export function App() {
  return <main>job-radar</main>;
}
```

`src/main.tsx`:

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource/jetbrains-mono";
import "@/shared/styles/theme.css";
import { App } from "@/app/App";

const root = document.getElementById("root");
if (!root) throw new Error("#root element missing from index.html");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

`src-tauri/Cargo.toml`:

```toml
[package]
name = "job-radar-desktop"
version = "0.1.0"
description = "Local-first job search tracker"
license = "MIT"
edition = "2024"

[lib]
# The _lib suffix keeps the lib name distinct from the bin (cargo#8519 on Windows).
name = "job_radar_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.7", features = [] }

[dependencies]
tauri = { version = "2.12", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
ts-rs = "12"

[profile.release]
codegen-units = 1
lto = true
opt-level = 3
panic = "abort"
strip = true
```

`src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "job-radar",
  "version": "0.1.0",
  "identifier": "dev.nandomoreira.jobradar",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "job-radar",
        "width": 1600,
        "height": 1000,
        "minWidth": 1280,
        "minHeight": 720,
        "maximized": true,
        "decorations": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: asset:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Main window: core defaults plus the controls of the custom title bar",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging"
  ]
}
```

`src-tauri/src/lib.rs`:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`src-tauri/src/main.rs`:

```rust
// Prevents an extra console window on Windows in release, do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    job_radar_desktop_lib::run()
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS (`✓ App renders the app name`, tsc and eslint silent, cargo `test result: ok`)

Manual: `pnpm tauri dev` opens a frameless, maximized window showing "job-radar". If the window is blank on Wayland, retry with `WEBKIT_DISABLE_DMABUF_RENDERER=1 pnpm tauri dev` and note it in STATE.md.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 2: Terminal Noir theme, glass/flat visual mode and WebKitGTK spike

**Requirement:** REQ-002
**Files:**

- Create: `src/shared/lib/visual-mode.ts`
- Create: `src/shared/assets/noir-skyline.png`
- Create: `src/shared/ui/button.tsx`
- Create: `src/shared/ui/badge.tsx`
- Create: `src/shared/ui/dialog.tsx`
- Create: `src/shared/ui/dropdown-menu.tsx`
- Create: `src/shared/ui/input.tsx`
- Modify: `src/shared/styles/theme.css`
- Modify: `src/main.tsx`
- Modify: `src/app/App.tsx`
- Test: `src/shared/lib/visual-mode.test.ts`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

```ts
// src/shared/lib/visual-mode.test.ts
import { describe, expect, it } from "vitest";
import { applyVisualMode, resolveVisualMode } from "./visual-mode";

describe("resolveVisualMode", () => {
  it("prefers the stored mode", () => {
    expect(resolveVisualMode("flat", "glass")).toBe("flat");
  });

  it("falls back to the env default", () => {
    expect(resolveVisualMode("bogus", "flat")).toBe("flat");
  });

  it("defaults to glass", () => {
    expect(resolveVisualMode(null, undefined)).toBe("glass");
  });
});

describe("applyVisualMode", () => {
  it("sets data-visual on the root", () => {
    const root = document.createElement("html");
    applyVisualMode("flat", root);
    expect(root.dataset.visual).toBe("flat");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/shared/lib/visual-mode.test.ts`
Expected: FAIL with "Failed to resolve import "./visual-mode""

**Step 3: Write minimal implementation**

`src/shared/lib/visual-mode.ts`:

```ts
export type VisualMode = "glass" | "flat";

export const VISUAL_MODE_KEY = "job-radar.visual-mode";

const isVisualMode = (value: unknown): value is VisualMode =>
  value === "glass" || value === "flat";

/**
 * Glass needs backdrop-filter, which WebKitGTK may render slowly; flat is the
 * fallback. A stored choice wins over the build-time default.
 */
export function resolveVisualMode(
  stored: string | null,
  envDefault: string | undefined,
): VisualMode {
  if (isVisualMode(stored)) return stored;
  if (isVisualMode(envDefault)) return envDefault;
  return "glass";
}

export function readStoredVisualMode(): string | null {
  try {
    return localStorage.getItem(VISUAL_MODE_KEY);
  } catch {
    return null;
  }
}

export function applyVisualMode(
  mode: VisualMode,
  root: HTMLElement = document.documentElement,
): void {
  root.dataset.visual = mode;
}
```

Background art and shadcn primitives:

```bash
mkdir -p src/shared/assets
cp ~/.dotfiles/wallpapers/terminal-noir-noir.png src/shared/assets/noir-skyline.png
pnpm dlx shadcn@4.23.0 add button badge dialog dropdown-menu input --yes
```

If shadcn rewrote `src/shared/styles/theme.css`, replace it with the content below (the Noir values must win). If the generated `DialogContent` has no `showCloseButton` prop, add one (`showCloseButton = true`, rendering the built-in close button only when true); Tasks 12-13 pass `showCloseButton={false}` to use translated close buttons.

`src/shared/styles/theme.css`:

```css
@import "tailwindcss";
@import "tw-animate-css";

@theme {
  --font-sans: "Inter Variable", ui-sans-serif, system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
  --font-serif: ui-serif, Georgia, serif;
  --color-noir-bg: #0b0b0b;
  --color-noir-sidebar: #0e0e10;
  --color-noir-panel: #111113;
  --color-noir-card: #16161a;
  --color-noir-border: #26262b;
  --color-noir-text: #ededed;
  --color-noir-muted: #8b8b94;
  --color-noir-amber: #f5a524;
  --color-noir-cream: #f7d9a8;
  --color-noir-teal: #2dd4a7;
  --color-noir-red: #e5484d;
}

/* shadcn semantic tokens mapped to the Terminal Noir palette (dark only). */
:root {
  color-scheme: dark;
  --radius: 0.5rem;
  --background: #0b0b0b;
  --foreground: #ededed;
  --card: #16161a;
  --card-foreground: #ededed;
  --popover: #16161a;
  --popover-foreground: #ededed;
  --primary: #f5a524;
  --primary-foreground: #0b0b0b;
  --secondary: #1c1c1f;
  --secondary-foreground: #ededed;
  --muted: #1c1c1f;
  --muted-foreground: #8b8b94;
  --accent: #1c1c1f;
  --accent-foreground: #ededed;
  --destructive: #e5484d;
  --border: #26262b;
  --input: #26262b;
  --ring: #f5a524;
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --radius-sm: calc(var(--radius) - 4px);
  --radius-md: calc(var(--radius) - 2px);
  --radius-lg: var(--radius);
  --radius-xl: calc(var(--radius) + 4px);
}

html,
body,
#root {
  height: 100%;
}

body {
  margin: 0;
  background: var(--color-noir-bg);
  color: var(--color-noir-text);
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;
}

.noir-backdrop {
  background: var(--color-noir-bg) url("../assets/noir-skyline.png") center bottom / cover no-repeat;
}

.noir-panel {
  background: var(--color-noir-panel);
  border: 1px solid var(--color-noir-border);
}

/* Glass: translucent towards the bottom so the skyline shows through. */
:root[data-visual="glass"] .noir-panel {
  background: linear-gradient(to bottom, rgb(17 17 19 / 0.85), rgb(17 17 19 / 0.35));
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

/* Flat: no blur; the skyline only shows in the gaps between panels. */
:root[data-visual="flat"] .noir-panel {
  background: rgb(17 17 19 / 0.92);
}

@media (prefers-reduced-transparency: reduce) {
  :root[data-visual="glass"] .noir-panel {
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    background: rgb(17 17 19 / 0.92);
  }
}

.noir-card {
  background: var(--color-noir-card);
  border: 1px solid var(--color-noir-border);
}

.noir-card:hover,
.noir-card:focus-within {
  border-color: var(--color-noir-amber);
}
```

(`tw-animate-css` comes with the shadcn add; if it is missing, `pnpm add tw-animate-css`.)

`src/main.tsx` (apply the mode before the first render):

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource/jetbrains-mono";
import "@/shared/styles/theme.css";
import { App } from "@/app/App";
import { applyVisualMode, readStoredVisualMode, resolveVisualMode } from "@/shared/lib/visual-mode";

applyVisualMode(resolveVisualMode(readStoredVisualMode(), import.meta.env.VITE_VISUAL_MODE));

const root = document.getElementById("root");
if (!root) throw new Error("#root element missing from index.html");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

`src/app/App.tsx` (temporary spike layout, replaced in Task 8):

```tsx
const SPIKE_COLUMNS = 6;
const SPIKE_CARDS = 4;

export function App() {
  return (
    <main className="noir-backdrop flex h-full flex-col gap-4 p-6">
      <h1 className="text-lg font-semibold">job-radar</h1>
      <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto">
        {Array.from({ length: SPIKE_COLUMNS }, (_, column) => (
          <section key={column} className="noir-panel flex flex-col gap-3 rounded-xl p-3">
            {Array.from({ length: SPIKE_CARDS }, (_, row) => (
              <div key={row} className="noir-card h-28 rounded-lg" />
            ))}
          </section>
        ))}
      </div>
    </main>
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ resolveVisualMode defaults to glass` and the other three cases)

**Step 4b: WebKitGTK spike on Niri (manual, decides the default mode)**

1. `pnpm tauri dev`: the skyline must be visible below the panels; resize and scroll the columns; watch for stutter or a blank window.
2. `VITE_VISUAL_MODE=flat pnpm tauri dev`: compare.
3. Blank window → retry with `WEBKIT_DISABLE_DMABUF_RENDERER=1` and record it.
4. If glass stutters, change the final `return "glass"` in `resolveVisualMode` to `return "flat"` and swap the `defaults to glass` test to expect `flat` (and rename it). The user accepted a borders-and-colors interface without blur.
5. Record the outcome (mode chosen, env workaround if any) in `docs/specs/board-prototype/STATE.md` under Session Log.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 3: Rust domain types with ts-rs bindings

**Requirement:** REQ-003
**Files:**

- Create: `src-tauri/src/domain.rs`
- Create: `src-tauri/.cargo/config.toml`
- Create: `src/shared/bindings/BoardCard.ts`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/domain.rs` (inline `#[cfg(test)]` module)

**Team:**

- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

Add `pub mod domain;` at the top of `src-tauri/src/lib.rs`, then create `src-tauri/src/domain.rs` with only the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stage_serializes_as_snake_case() {
        assert_eq!(serde_json::to_value(Stage::InProcess).unwrap(), json!("in_process"));
    }

    #[test]
    fn outcome_serializes_as_snake_case() {
        assert_eq!(serde_json::to_value(Outcome::NoResponse).unwrap(), json!("no_response"));
    }

    #[test]
    fn salary_keeps_a_missing_regime_as_null() {
        let salary = Salary { pj: Some(28_000), clt: None };
        assert_eq!(serde_json::to_value(salary).unwrap(), json!({ "pj": 28000, "clt": null }));
    }

    #[test]
    fn artifacts_use_camel_case() {
        let artifacts = Artifacts { cv: true, coverage: false, interview_prep: true };
        assert_eq!(
            serde_json::to_value(artifacts).unwrap(),
            json!({ "cv": true, "coverage": false, "interviewPrep": true })
        );
    }

    #[test]
    fn stage_order_matches_the_board() {
        assert_eq!(Stage::ALL.first(), Some(&Stage::Inbox));
        assert_eq!(Stage::ALL.last(), Some(&Stage::Closed));
        assert_eq!(Stage::ALL.len(), 6);
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test --manifest-path src-tauri/Cargo.toml domain`
Expected: FAIL with "cannot find type `Salary` in this scope" (and the other domain types)

**Step 3: Write minimal implementation**

`src-tauri/.cargo/config.toml` (paths are relative to `src-tauri/`, the parent of `.cargo`):

```toml
[env]
TS_RS_EXPORT_DIR = { value = "../src/shared/bindings", relative = true }
```

Prepend to `src-tauri/src/domain.rs` (above the tests):

```rust
//! Board domain shared with the frontend. `cargo test` exports these types to
//! `src/shared/bindings` through ts-rs, so the React side never redeclares them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Stage {
    Inbox,
    Triage,
    Applied,
    InProcess,
    Offer,
    Closed,
}

impl Stage {
    /// Column order on the board, left to right.
    pub const ALL: [Stage; 6] = [
        Stage::Inbox,
        Stage::Triage,
        Stage::Applied,
        Stage::InProcess,
        Stage::Offer,
        Stage::Closed,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Outcome {
    Hired,
    Rejected,
    Discarded,
    NoResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Target {
    AiLlm,
    FullStack,
    TechLead,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WorkModel {
    Remote,
    Hybrid,
    Onsite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EventKind {
    Found,
    Applied,
    Response,
    Interview,
    Offer,
    Closed,
}

/// Negotiated monthly pay in BRL. Either regime may be missing; both missing
/// means nothing was negotiated yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Salary {
    pub pj: Option<u32>,
    pub clt: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Artifacts {
    pub cv: bool,
    pub coverage: bool,
    pub interview_prep: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineEvent {
    /// ISO date, `YYYY-MM-DD`.
    pub at: String,
    pub kind: EventKind,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BoardCard {
    pub id: String,
    pub title: String,
    pub company: String,
    pub source: String,
    pub target: Target,
    pub work_model: WorkModel,
    pub location: Option<String>,
    pub score: u8,
    pub stage: Stage,
    /// Set only when `stage` is `Closed`.
    pub outcome: Option<Outcome>,
    /// Free label for the current interview step ("Técnica", "Case").
    pub process_step: Option<String>,
    pub salary: Option<Salary>,
    pub artifacts: Artifacts,
    pub last_activity: Option<String>,
    pub next_action: Option<String>,
    /// Name of the AI CLI working on this card, if any.
    pub ai_running: Option<String>,
    pub unread: bool,
    pub timeline: Vec<TimelineEvent>,
}
```

**Step 4: Run test to verify it passes**
Run: `cargo test --manifest-path src-tauri/Cargo.toml && [ -f src/shared/bindings/BoardCard.ts ] && pnpm typecheck`
Expected: PASS (5 domain tests plus the ts-rs `export_bindings_*` tests ok; `src/shared/bindings/*.ts` exists and type-checks). Commit the generated bindings.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 4: `Store` trait, move rules and `MockStore` seed

**Requirement:** REQ-004, REQ-005
**Files:**

- Create: `src-tauri/src/store.rs`
- Create: `src-tauri/src/mock_data.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/store.rs`, `src-tauri/src/mock_data.rs` (inline test modules)

**Team:**

- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

Add `pub mod mock_data;` and `pub mod store;` to `src-tauri/src/lib.rs`. Create both files with only their tests.

`src-tauri/src/store.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_data::sample_cards;

    fn store() -> MockStore {
        MockStore::new(sample_cards())
    }

    #[test]
    fn lists_every_seeded_card() {
        assert_eq!(store().list_cards().unwrap().len(), sample_cards().len());
    }

    #[test]
    fn moving_updates_the_stage_for_the_session() {
        let store = store();
        let moved = store.move_card("c01", Stage::Triage, None).unwrap();
        assert_eq!(moved.stage, Stage::Triage);
        let listed = store.list_cards().unwrap();
        assert_eq!(listed.iter().find(|c| c.id == "c01").unwrap().stage, Stage::Triage);
    }

    #[test]
    fn moving_to_closed_requires_an_outcome() {
        assert_eq!(
            store().move_card("c01", Stage::Closed, None),
            Err(StoreError::OutcomeRequired)
        );
    }

    #[test]
    fn an_outcome_outside_closed_is_rejected() {
        assert_eq!(
            store().move_card("c01", Stage::Offer, Some(Outcome::Hired)),
            Err(StoreError::UnexpectedOutcome)
        );
    }

    #[test]
    fn an_unknown_card_is_not_found() {
        assert_eq!(
            store().move_card("nope", Stage::Triage, None),
            Err(StoreError::NotFound("nope".into()))
        );
    }

    #[test]
    fn moving_out_of_closed_clears_the_outcome() {
        let store = store();
        store.move_card("c01", Stage::Closed, Some(Outcome::Discarded)).unwrap();
        let reopened = store.move_card("c01", Stage::Triage, None).unwrap();
        assert_eq!(reopened.outcome, None);
    }
}
```

`src-tauri/src/mock_data.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn seeds_every_board_stage() {
        let cards = sample_cards();
        for stage in Stage::ALL {
            assert!(cards.iter().any(|c| c.stage == stage), "no card in {stage:?}");
        }
    }

    #[test]
    fn closed_cards_carry_an_outcome() {
        for card in sample_cards() {
            assert_eq!(card.stage == Stage::Closed, card.outcome.is_some(), "{}", card.id);
        }
    }

    #[test]
    fn cards_before_applying_have_no_salary() {
        for card in sample_cards() {
            if matches!(card.stage, Stage::Inbox | Stage::Triage) {
                assert!(card.salary.is_none(), "{}", card.id);
            }
        }
    }

    #[test]
    fn ids_are_unique() {
        let cards = sample_cards();
        let ids: HashSet<_> = cards.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids.len(), cards.len());
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test --manifest-path src-tauri/Cargo.toml store mock_data`
Expected: FAIL with "cannot find function `sample_cards` in this scope" and "cannot find type `MockStore` in this scope"

**Step 3: Write minimal implementation**

Prepend to `src-tauri/src/store.rs`:

```rust
//! Persistence boundary. Commands and the UI only see `Store`; the SQLite
//! (default) and Postgres (self-hosted) backends implement it in later plans.

use std::sync::Mutex;

use crate::domain::{BoardCard, Outcome, Stage};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    #[error("card not found: {0}")]
    NotFound(String),
    #[error("moving to closed requires an outcome")]
    OutcomeRequired,
    #[error("only closed cards take an outcome")]
    UnexpectedOutcome,
    #[error("store lock poisoned")]
    Poisoned,
}

pub trait Store: Send + Sync {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError>;
    fn move_card(
        &self,
        id: &str,
        stage: Stage,
        outcome: Option<Outcome>,
    ) -> Result<BoardCard, StoreError>;
}

/// Checks the stage/outcome pair before any backend touches its data.
pub fn validate_move(stage: Stage, outcome: Option<Outcome>) -> Result<(), StoreError> {
    match (stage, outcome) {
        (Stage::Closed, None) => Err(StoreError::OutcomeRequired),
        (Stage::Closed, Some(_)) | (_, None) => Ok(()),
        (_, Some(_)) => Err(StoreError::UnexpectedOutcome),
    }
}

/// In-memory store seeded with fictional cards; changes last for the session.
pub struct MockStore {
    cards: Mutex<Vec<BoardCard>>,
}

impl MockStore {
    pub fn new(cards: Vec<BoardCard>) -> Self {
        Self { cards: Mutex::new(cards) }
    }
}

impl Store for MockStore {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError> {
        Ok(self.cards.lock().map_err(|_| StoreError::Poisoned)?.clone())
    }

    fn move_card(
        &self,
        id: &str,
        stage: Stage,
        outcome: Option<Outcome>,
    ) -> Result<BoardCard, StoreError> {
        validate_move(stage, outcome)?;
        let mut cards = self.cards.lock().map_err(|_| StoreError::Poisoned)?;
        let card = cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_owned()))?;
        card.stage = stage;
        card.outcome = outcome;
        Ok(card.clone())
    }
}
```

Prepend to `src-tauri/src/mock_data.rs` (fictional companies only):

```rust
//! Fictional seed for the prototype board. Dates are fixed around 2026-11-02.

use crate::domain::{
    Artifacts, BoardCard, EventKind, Outcome, Salary, Stage, Target, TimelineEvent, WorkModel,
};

#[allow(clippy::too_many_arguments)]
fn card(
    id: &str,
    title: &str,
    company: &str,
    source: &str,
    target: Target,
    work_model: WorkModel,
    score: u8,
    stage: Stage,
) -> BoardCard {
    BoardCard {
        id: id.into(),
        title: title.into(),
        company: company.into(),
        source: source.into(),
        target,
        work_model,
        location: None,
        score,
        stage,
        outcome: None,
        process_step: None,
        salary: None,
        artifacts: Artifacts { cv: false, coverage: false, interview_prep: false },
        last_activity: None,
        next_action: None,
        ai_running: None,
        unread: false,
        timeline: Vec::new(),
    }
}

fn ev(at: &str, kind: EventKind, label: &str) -> TimelineEvent {
    TimelineEvent { at: at.into(), kind, label: label.into() }
}

fn pay(pj: Option<u32>, clt: Option<u32>) -> Option<Salary> {
    Some(Salary { pj, clt })
}

fn text(value: &str) -> Option<String> {
    Some(value.into())
}

const ALL_ARTIFACTS: Artifacts = Artifacts { cv: true, coverage: true, interview_prep: true };

pub fn sample_cards() -> Vec<BoardCard> {
    use EventKind as E;
    use Stage as S;
    use Target as T;
    use WorkModel as W;

    vec![
        BoardCard {
            unread: true,
            last_activity: text("Nova vaga"),
            timeline: vec![ev("2026-11-02", E::Found, "Encontrada na Greenhouse")],
            ..card("c01", "Staff Engineer – LLM Platform", "Lumen Labs", "Greenhouse", T::AiLlm, W::Remote, 94, S::Inbox)
        },
        BoardCard {
            unread: true,
            last_activity: text("Nova vaga"),
            timeline: vec![ev("2026-11-02", E::Found, "Encontrada na Ashby")],
            ..card("c02", "Senior AI Engineer", "Cobalto AI", "Ashby", T::AiLlm, W::Remote, 88, S::Inbox)
        },
        BoardCard {
            last_activity: text("Adicionada ontem"),
            timeline: vec![ev("2026-11-01", E::Found, "Encontrada na Greenhouse")],
            ..card("c03", "Principal MLOps Engineer", "Synthetica", "Greenhouse", T::AiLlm, W::Remote, 90, S::Inbox)
        },
        BoardCard {
            location: text("São Paulo"),
            ai_running: text("claude"),
            timeline: vec![ev("2026-10-31", E::Found, "Encontrada na Gupy")],
            ..card("c04", "Tech Lead Full Stack", "Orbita Pay", "Gupy", T::TechLead, W::Hybrid, 82, S::Triage)
        },
        BoardCard {
            last_activity: text("Triada há 3 dias"),
            timeline: vec![ev("2026-10-30", E::Found, "Encontrada na Lever")],
            ..card("c05", "Engenheiro de Dados Sênior", "Nébula Data", "Lever", T::Data, W::Remote, 79, S::Triage)
        },
        BoardCard {
            last_activity: text("Triada há 2 dias"),
            timeline: vec![ev("2026-10-31", E::Found, "Encontrada na Workable")],
            ..card("c06", "Staff Backend Engineer", "Veloce Tech", "Workable", T::FullStack, W::Remote, 84, S::Triage)
        },
        BoardCard {
            salary: pay(Some(28_000), Some(21_000)),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            last_activity: text("Candidatura enviada há 3 dias"),
            timeline: vec![
                ev("2026-10-25", E::Found, "Alerta do Gmail"),
                ev("2026-10-30", E::Applied, "Candidatura enviada"),
            ],
            ..card("c07", "Lead AI Architect", "Vértice Saúde", "Gmail", T::AiLlm, W::Remote, 91, S::Applied)
        },
        BoardCard {
            salary: pay(Some(26_000), None),
            artifacts: Artifacts { cv: true, coverage: false, interview_prep: false },
            last_activity: text("Visualizado pelo RH há 1 dia"),
            timeline: vec![
                ev("2026-10-27", E::Found, "Encontrada no GitHub"),
                ev("2026-10-29", E::Applied, "Candidatura enviada"),
                ev("2026-11-01", E::Response, "Visualizado pelo RH"),
            ],
            ..card("c08", "Senior Backend Engineer", "Horizon Tech", "GitHub", T::FullStack, W::Remote, 85, S::Applied)
        },
        BoardCard {
            salary: pay(Some(32_000), None),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            last_activity: text("Candidatura enviada há 5 dias"),
            timeline: vec![
                ev("2026-10-26", E::Found, "Encontrada na Greenhouse"),
                ev("2026-10-28", E::Applied, "Candidatura enviada"),
            ],
            ..card("c09", "Tech Lead Distributed Systems", "CloudScale", "Greenhouse", T::TechLead, W::Remote, 87, S::Applied)
        },
        BoardCard {
            process_step: text("Técnica"),
            salary: pay(Some(34_000), Some(26_000)),
            artifacts: ALL_ARTIFACTS,
            next_action: text("Técnica amanhã 15:00"),
            timeline: vec![
                ev("2026-10-20", E::Applied, "Candidatura enviada"),
                ev("2026-10-23", E::Response, "RH respondeu pelo Gmail"),
                ev("2026-10-28", E::Interview, "Entrevista com RH"),
            ],
            ..card("c10", "Staff ML Engineer", "Quasar Bio", "Gmail", T::AiLlm, W::Remote, 92, S::InProcess)
        },
        BoardCard {
            location: text("São Paulo"),
            process_step: text("Case"),
            salary: pay(Some(30_000), None),
            artifacts: Artifacts { cv: true, coverage: true, interview_prep: false },
            next_action: text("Aguardando retorno do case"),
            timeline: vec![
                ev("2026-10-18", E::Applied, "Candidatura enviada"),
                ev("2026-10-22", E::Response, "Convite para o case"),
                ev("2026-10-27", E::Interview, "Case entregue"),
            ],
            ..card("c11", "Principal Systems Architect", "Nexus Infra", "Gupy", T::TechLead, W::Hybrid, 89, S::InProcess)
        },
        BoardCard {
            process_step: text("RH"),
            salary: pay(Some(29_000), Some(22_000)),
            artifacts: Artifacts { cv: true, coverage: false, interview_prep: true },
            next_action: text("Feedback em 2 dias"),
            timeline: vec![
                ev("2026-10-21", E::Applied, "Candidatura enviada"),
                ev("2026-10-24", E::Response, "RH respondeu"),
            ],
            ..card("c12", "Lead Platform Engineer", "KubeWorks", "Lever", T::FullStack, W::Remote, 86, S::InProcess)
        },
        BoardCard {
            process_step: text("Proposta"),
            salary: pay(Some(40_000), Some(31_000)),
            artifacts: ALL_ARTIFACTS,
            next_action: text("Proposta expira em 4 dias"),
            timeline: vec![
                ev("2026-10-10", E::Applied, "Candidatura enviada"),
                ev("2026-10-13", E::Response, "RH respondeu"),
                ev("2026-10-17", E::Interview, "Entrevista técnica"),
                ev("2026-10-31", E::Offer, "Carta proposta recebida"),
            ],
            ..card("c13", "Principal AI Lead", "Zenith Core", "Greenhouse", T::AiLlm, W::Remote, 96, S::Offer)
        },
        BoardCard {
            outcome: Some(Outcome::Hired),
            salary: pay(Some(42_000), Some(33_000)),
            artifacts: ALL_ARTIFACTS,
            last_activity: text("Contrato assinado"),
            timeline: vec![
                ev("2026-09-20", E::Applied, "Candidatura enviada"),
                ev("2026-09-24", E::Response, "RH respondeu"),
                ev("2026-10-08", E::Offer, "Proposta recebida"),
                ev("2026-10-12", E::Closed, "Contrato assinado"),
            ],
            ..card("c14", "Head of AI Architecture", "Atlas Cloud", "Ashby", T::AiLlm, W::Remote, 93, S::Closed)
        },
        BoardCard {
            location: text("São Paulo"),
            outcome: Some(Outcome::Rejected),
            salary: pay(Some(25_000), None),
            last_activity: text("Proposta recusada: abaixo do alvo"),
            timeline: vec![
                ev("2026-10-01", E::Applied, "Candidatura enviada"),
                ev("2026-10-06", E::Response, "RH respondeu"),
                ev("2026-10-15", E::Closed, "Proposta recusada"),
            ],
            ..card("c15", "Staff Data Engineer", "Solaris Fin", "Gupy", T::Data, W::Hybrid, 81, S::Closed)
        },
        BoardCard {
            outcome: Some(Outcome::NoResponse),
            salary: pay(None, Some(18_000)),
            last_activity: text("Sem resposta há 21 dias"),
            timeline: vec![
                ev("2026-10-05", E::Applied, "Candidatura enviada"),
                ev("2026-10-26", E::Closed, "Encerrada sem resposta"),
            ],
            ..card("c16", "Backend Tech Lead", "Prisma Retail", "Gupy", T::TechLead, W::Onsite, 72, S::Closed)
        },
        BoardCard {
            outcome: Some(Outcome::Discarded),
            last_activity: text("Descartada na triagem"),
            timeline: vec![
                ev("2026-10-20", E::Found, "Encontrada na ProgramaThor"),
                ev("2026-10-21", E::Closed, "Descartada na triagem"),
            ],
            ..card("c17", "Desenvolvedor Full Stack Sênior", "Delta Commerce", "ProgramaThor", T::FullStack, W::Remote, 70, S::Closed)
        },
    ]
}
```

**Step 4: Run test to verify it passes**
Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS (6 store tests and 4 mock_data tests ok; clippy clean)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 5: Tauri commands `list_board` and `move_card`

**Requirement:** REQ-004, REQ-005
**Files:**

- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands.rs` (inline test module)

**Team:**

- Track: backend
- Implementer: backend-engineer (skills: rust-cli-conventions)
- Reviewers: business-logic-reviewer, security-reviewer, type-safety-reviewer, test-reviewer

**Step 1: Write the failing test**

Add `pub mod commands;` to `src-tauri/src/lib.rs`; create `src-tauri/src/commands.rs` with the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_data::sample_cards;
    use crate::store::MockStore;

    fn state() -> AppState {
        AppState { store: Box::new(MockStore::new(sample_cards())) }
    }

    #[test]
    fn list_board_returns_the_seeded_cards() {
        assert_eq!(list_board_impl(&state()).unwrap().len(), sample_cards().len());
    }

    #[test]
    fn move_card_persists_the_new_stage() {
        let state = state();
        move_card_impl(&state, "c02", Stage::Triage, None).unwrap();
        let cards = list_board_impl(&state).unwrap();
        assert_eq!(cards.iter().find(|c| c.id == "c02").unwrap().stage, Stage::Triage);
    }

    #[test]
    fn move_card_rejects_an_empty_id() {
        assert_eq!(
            move_card_impl(&state(), "", Stage::Triage, None),
            Err("invalid card id".to_string())
        );
    }

    #[test]
    fn move_card_rejects_an_oversized_id() {
        let id = "x".repeat(MAX_ID_LEN + 1);
        assert_eq!(
            move_card_impl(&state(), &id, Stage::Triage, None),
            Err("invalid card id".to_string())
        );
    }

    #[test]
    fn move_card_reports_the_store_rule() {
        assert_eq!(
            move_card_impl(&state(), "c01", Stage::Closed, None),
            Err("moving to closed requires an outcome".to_string())
        );
    }
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test --manifest-path src-tauri/Cargo.toml commands`
Expected: FAIL with "cannot find struct, variant or union type `AppState` in this scope"

**Step 3: Write minimal implementation**

Prepend to `src-tauri/src/commands.rs`:

```rust
//! IPC surface. Command bodies live in `*_impl` functions so tests call them
//! without a running Tauri app; the wrappers only unpack `State`.

use tauri::State;

use crate::domain::{BoardCard, Outcome, Stage};
use crate::store::Store;

pub struct AppState {
    pub store: Box<dyn Store>,
}

/// Ids are short opaque strings; anything else from the webview is rejected.
pub const MAX_ID_LEN: usize = 64;

fn check_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err("invalid card id".into());
    }
    Ok(())
}

pub fn list_board_impl(state: &AppState) -> Result<Vec<BoardCard>, String> {
    state.store.list_cards().map_err(|e| e.to_string())
}

pub fn move_card_impl(
    state: &AppState,
    id: &str,
    stage: Stage,
    outcome: Option<Outcome>,
) -> Result<BoardCard, String> {
    check_id(id)?;
    state.store.move_card(id, stage, outcome).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_board(state: State<'_, AppState>) -> Result<Vec<BoardCard>, String> {
    list_board_impl(&state)
}

#[tauri::command]
pub fn move_card(
    state: State<'_, AppState>,
    id: String,
    stage: Stage,
    outcome: Option<Outcome>,
) -> Result<BoardCard, String> {
    move_card_impl(&state, &id, stage, outcome)
}
```

`src-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod domain;
pub mod mock_data;
pub mod store;

use commands::AppState;
use store::MockStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { store: Box::new(MockStore::new(mock_data::sample_cards())) })
        .invoke_handler(tauri::generate_handler![commands::list_board, commands::move_card])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Run test to verify it passes**
Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS (5 command tests ok, all earlier tests still ok, clippy clean)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 6: i18n with pt-BR (default) and en

**Requirement:** REQ-013
**Files:**

- Create: `src/app/i18n.ts`
- Create: `src/shared/locales/pt-BR.json`
- Create: `src/shared/locales/en.json`
- Modify: `src/test/setup.ts`
- Modify: `src/main.tsx`
- Test: `src/shared/locales/locales.test.ts`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

```ts
// src/shared/locales/locales.test.ts
import { describe, expect, it } from "vitest";
import i18n from "@/app/i18n";
import en from "./en.json";
import ptBR from "./pt-BR.json";

function keys(tree: object, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === "object" && value !== null
      ? keys(value as object, `${prefix}${key}.`)
      : [`${prefix}${key}`],
  );
}

describe("locales", () => {
  it("have the same keys in pt-BR and en", () => {
    expect(keys(en).sort()).toEqual(keys(ptBR).sort());
  });

  it("default to pt-BR", () => {
    expect(i18n.language).toBe("pt-BR");
    expect(i18n.t("stage.inbox")).toBe("Caixa de entrada");
  });

  it("translate to English", () => {
    expect(i18n.getFixedT("en")("stage.inbox")).toBe("Inbox");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/shared/locales/locales.test.ts`
Expected: FAIL with "Failed to resolve import "@/app/i18n""

**Step 3: Write minimal implementation**

`src/app/i18n.ts`:

```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "@/shared/locales/en.json";
import ptBR from "@/shared/locales/pt-BR.json";

void i18n.use(initReactI18next).init({
  resources: { "pt-BR": { translation: ptBR }, en: { translation: en } },
  lng: "pt-BR",
  fallbackLng: "pt-BR",
  // React already escapes rendered text.
  interpolation: { escapeValue: false },
  initAsync: false,
});

export default i18n;
```

`src/shared/locales/pt-BR.json`:

```json
{
  "app": { "name": "job-radar", "signature": "terminal noir" },
  "titlebar": { "minimize": "Minimizar", "maximize": "Maximizar", "close": "Fechar" },
  "nav": {
    "label": "Navegação principal",
    "board": "Board",
    "profile": "Meu perfil",
    "resumes": "Meus currículos",
    "prep": "Preparação",
    "settings": "Configuração",
    "pending": "{{count}} pendentes"
  },
  "crawler": {
    "active": "Crawler ativo",
    "lastRun": "Última varredura: hoje {{time}} · {{total}} vagas · {{fresh}} novas"
  },
  "placeholder": { "body": "Esta tela chega nos próximos planos." },
  "board": {
    "title": "Board",
    "loading": "Carregando vagas…",
    "error": "Não foi possível carregar as vagas.",
    "moveError": "Não foi possível mover a vaga.",
    "empty": "Nenhuma vaga aqui.",
    "count_one": "{{count}} vaga",
    "count_other": "{{count}} vagas"
  },
  "filters": {
    "search": "Buscar vaga, empresa…",
    "source": "Fonte",
    "target": "Alvo",
    "all": "Todas",
    "selected_one": "{{count}} selecionado",
    "selected_other": "{{count}} selecionados",
    "minScore": "Score mín.",
    "syncGmail": "Sincronizar Gmail"
  },
  "metrics": {
    "appliedWeek": "Candidaturas na semana",
    "responseRate": "Taxa de resposta",
    "interviews": "Em processo",
    "avgResponse": "Tempo médio até resposta",
    "days": "{{value}}d",
    "none": "—"
  },
  "stage": {
    "inbox": "Caixa de entrada",
    "triage": "Triagem",
    "applied": "Candidatado",
    "in_process": "Em processo",
    "offer": "Oferta",
    "closed": "Encerradas"
  },
  "outcome": {
    "hired": "Contratado",
    "rejected": "Recusada",
    "discarded": "Descartada",
    "no_response": "Sem resposta"
  },
  "target": { "ai_llm": "AI/LLM", "full_stack": "Full Stack", "tech_lead": "Tech Lead", "data": "Dados" },
  "workModel": { "remote": "Remoto", "hybrid": "Híbrido", "onsite": "Presencial" },
  "card": {
    "scoreLabel": "Score",
    "unread": "Não lida",
    "aiRunning": "{{cli}} trabalhando…",
    "open": "Abrir detalhes de {{title}}",
    "menu": "Ações de {{title}}",
    "artifacts": {
      "label": "Artefatos",
      "cv": "CV customizado",
      "coverage": "Revisão de cobertura",
      "interviewPrep": "Preparação de entrevista",
      "done": "pronto",
      "missing": "pendente"
    }
  },
  "salary": { "pj": "PJ", "clt": "CLT", "value": "R$ {{value}}k" },
  "move": {
    "to": "Mover para",
    "closeAs": "Encerrar como",
    "pickOutcome": "Como esta vaga foi encerrada?",
    "cancel": "Cancelar"
  },
  "dnd": {
    "picked": "{{title}} selecionada.",
    "over": "{{title}} sobre {{column}}.",
    "dropped": "{{title}} solta em {{column}}.",
    "cancelled": "Movimento de {{title}} cancelado."
  },
  "dialog": {
    "salary": "Salário negociado",
    "noSalary": "Ainda sem negociação.",
    "artifacts": "Artefatos",
    "timeline": "Linha do tempo",
    "generateCv": "Gerar CV customizado",
    "comingSoon": "Em breve",
    "close": "Fechar"
  }
}
```

`src/shared/locales/en.json`:

```json
{
  "app": { "name": "job-radar", "signature": "terminal noir" },
  "titlebar": { "minimize": "Minimize", "maximize": "Maximize", "close": "Close" },
  "nav": {
    "label": "Main navigation",
    "board": "Board",
    "profile": "My profile",
    "resumes": "My resumes",
    "prep": "Preparation",
    "settings": "Settings",
    "pending": "{{count}} pending"
  },
  "crawler": {
    "active": "Crawler active",
    "lastRun": "Last run: today {{time}} · {{total}} jobs · {{fresh}} new"
  },
  "placeholder": { "body": "This screen arrives in upcoming plans." },
  "board": {
    "title": "Board",
    "loading": "Loading jobs…",
    "error": "Could not load the jobs.",
    "moveError": "Could not move the job.",
    "empty": "No jobs here.",
    "count_one": "{{count}} job",
    "count_other": "{{count}} jobs"
  },
  "filters": {
    "search": "Search job, company…",
    "source": "Source",
    "target": "Target",
    "all": "All",
    "selected_one": "{{count}} selected",
    "selected_other": "{{count}} selected",
    "minScore": "Min. score",
    "syncGmail": "Sync Gmail"
  },
  "metrics": {
    "appliedWeek": "Applications this week",
    "responseRate": "Response rate",
    "interviews": "In process",
    "avgResponse": "Average time to response",
    "days": "{{value}}d",
    "none": "—"
  },
  "stage": {
    "inbox": "Inbox",
    "triage": "Triage",
    "applied": "Applied",
    "in_process": "In process",
    "offer": "Offer",
    "closed": "Closed"
  },
  "outcome": {
    "hired": "Hired",
    "rejected": "Rejected",
    "discarded": "Discarded",
    "no_response": "No response"
  },
  "target": { "ai_llm": "AI/LLM", "full_stack": "Full Stack", "tech_lead": "Tech Lead", "data": "Data" },
  "workModel": { "remote": "Remote", "hybrid": "Hybrid", "onsite": "On-site" },
  "card": {
    "scoreLabel": "Score",
    "unread": "Unread",
    "aiRunning": "{{cli}} working…",
    "open": "Open details of {{title}}",
    "menu": "Actions for {{title}}",
    "artifacts": {
      "label": "Artifacts",
      "cv": "Tailored CV",
      "coverage": "Coverage review",
      "interviewPrep": "Interview prep",
      "done": "done",
      "missing": "missing"
    }
  },
  "salary": { "pj": "PJ", "clt": "CLT", "value": "R$ {{value}}k" },
  "move": {
    "to": "Move to",
    "closeAs": "Close as",
    "pickOutcome": "How did this job close?",
    "cancel": "Cancel"
  },
  "dnd": {
    "picked": "Picked up {{title}}.",
    "over": "{{title}} is over {{column}}.",
    "dropped": "{{title}} dropped on {{column}}.",
    "cancelled": "Moving {{title}} was cancelled."
  },
  "dialog": {
    "salary": "Negotiated salary",
    "noSalary": "Not negotiated yet.",
    "artifacts": "Artifacts",
    "timeline": "Timeline",
    "generateCv": "Generate tailored CV",
    "comingSoon": "Coming soon",
    "close": "Close"
  }
}
```

Add `import "@/app/i18n";` as the first line of `src/test/setup.ts`, and after the CSS import in `src/main.tsx`.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck`
Expected: PASS (`✓ locales have the same keys in pt-BR and en`, `default to pt-BR`, `translate to English`)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 7: Board API client and `useBoard` hook

**Requirement:** REQ-004, REQ-005
**Files:**

- Create: `src/shared/api/board.ts`
- Create: `src/entities/job/model/use-board.ts`
- Create: `src/test/factories.ts`
- Test: `src/entities/job/model/use-board.test.ts`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, security-reviewer

**Step 1: Write the failing test**

`src/test/factories.ts`:

```ts
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { BoardApi } from "@/shared/api/board";

export function makeCard(overrides: Partial<BoardCard> = {}): BoardCard {
  return {
    id: "c01",
    title: "Staff Engineer",
    company: "Lumen Labs",
    source: "Greenhouse",
    target: "ai_llm",
    workModel: "remote",
    location: null,
    score: 94,
    stage: "inbox",
    outcome: null,
    processStep: null,
    salary: null,
    artifacts: { cv: false, coverage: false, interviewPrep: false },
    lastActivity: null,
    nextAction: null,
    aiRunning: null,
    unread: false,
    timeline: [],
    ...overrides,
  };
}

/** In-memory BoardApi that applies moves like the Rust MockStore. */
export function fakeApi(cards: BoardCard[]): BoardApi {
  let state = [...cards];
  return {
    list: async () => state,
    move: async (id, stage, outcome) => {
      const card = state.find((c) => c.id === id);
      if (!card) throw new Error(`card not found: ${id}`);
      const moved = { ...card, stage, outcome };
      state = state.map((c) => (c.id === id ? moved : c));
      return moved;
    },
  };
}
```

`src/entities/job/model/use-board.test.ts`:

```ts
import { renderHook, waitFor, act } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { fakeApi, makeCard } from "@/test/factories";
import { useBoard } from "./use-board";

describe("useBoard", () => {
  it("loads the cards from the api", async () => {
    const api = fakeApi([makeCard(), makeCard({ id: "c02" })]);
    const { result } = renderHook(() => useBoard(api));
    await waitFor(() => expect(result.current.status).toBe("ready"));
    expect(result.current.cards).toHaveLength(2);
  });

  it("reports an error when loading fails", async () => {
    const api = { ...fakeApi([]), list: async () => Promise.reject(new Error("boom")) };
    const { result } = renderHook(() => useBoard(api));
    await waitFor(() => expect(result.current.status).toBe("error"));
  });

  it("replaces the moved card with the store answer", async () => {
    const api = fakeApi([makeCard()]);
    const { result } = renderHook(() => useBoard(api));
    await waitFor(() => expect(result.current.status).toBe("ready"));
    await act(() => result.current.move("c01", "triage", null));
    expect(result.current.cards[0].stage).toBe("triage");
    expect(result.current.moveError).toBe(false);
  });

  it("flags a failed move and keeps the cards", async () => {
    const api = fakeApi([makeCard()]);
    const { result } = renderHook(() => useBoard(api));
    await waitFor(() => expect(result.current.status).toBe("ready"));
    await act(() => result.current.move("missing", "triage", null));
    expect(result.current.moveError).toBe(true);
    expect(result.current.cards[0].stage).toBe("inbox");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/entities/job/model/use-board.test.ts`
Expected: FAIL with "Failed to resolve import "@/shared/api/board""

**Step 3: Write minimal implementation**

`src/shared/api/board.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";

export type BoardApi = {
  list(): Promise<BoardCard[]>;
  move(id: string, stage: Stage, outcome: Outcome | null): Promise<BoardCard>;
};

/** Thin wrappers over the Rust commands in `src-tauri/src/commands.rs`. */
export const boardApi: BoardApi = {
  list: () => invoke<BoardCard[]>("list_board"),
  move: (id, stage, outcome) => invoke<BoardCard>("move_card", { id, stage, outcome }),
};
```

`src/entities/job/model/use-board.ts`:

```ts
import { useCallback, useEffect, useState } from "react";
import type { BoardApi } from "@/shared/api/board";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";

export type BoardStatus = "loading" | "ready" | "error";

export function useBoard(api: BoardApi) {
  const [cards, setCards] = useState<BoardCard[]>([]);
  const [status, setStatus] = useState<BoardStatus>("loading");
  const [moveError, setMoveError] = useState(false);

  useEffect(() => {
    let alive = true;
    api.list().then(
      (list) => {
        if (!alive) return;
        setCards(list);
        setStatus("ready");
      },
      () => {
        if (alive) setStatus("error");
      },
    );
    return () => {
      alive = false;
    };
  }, [api]);

  const move = useCallback(
    async (id: string, stage: Stage, outcome: Outcome | null) => {
      try {
        const updated = await api.move(id, stage, outcome);
        setCards((list) => list.map((c) => (c.id === id ? updated : c)));
        setMoveError(false);
      } catch {
        // The store rejected the move; the card stays where it was.
        setMoveError(true);
      }
    },
    [api],
  );

  return { cards, status, moveError, move };
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (4 `useBoard` tests)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 8: App shell (title bar, sidebar, placeholder pages)

**Requirement:** REQ-006
**Files:**

- Create: `src/shared/config/navigation.ts`
- Create: `src/widgets/app-shell/TitleBar.tsx`
- Create: `src/widgets/app-shell/Sidebar.tsx`
- Create: `src/widgets/app-shell/AppShell.tsx`
- Create: `src/pages/placeholder/PlaceholderPage.tsx`
- Modify: `src/app/App.tsx`
- Test: `src/app/App.test.tsx` (replaces the Task 1 test)
- Test: `src/widgets/app-shell/AppShell.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/widgets/app-shell/AppShell.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "./AppShell";

const controls = () => ({
  minimize: vi.fn(async () => {}),
  toggleMaximize: vi.fn(async () => {}),
  close: vi.fn(async () => {}),
});

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
    expect(screen.getByRole("button", { name: /Board/ })).not.toHaveAttribute("aria-current");
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
});
```

`src/app/App.test.tsx` (replaces the Task 1 test):

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => []) }));

describe("App", () => {
  it("shows a placeholder for pages not built yet", async () => {
    render(<App />);
    await userEvent.click(screen.getByRole("button", { name: /Meus currículos/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Meus currículos" })).toBeInTheDocument();
    expect(screen.getByText("Esta tela chega nos próximos planos.")).toBeInTheDocument();
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/widgets/app-shell src/app`
Expected: FAIL with "Failed to resolve import "./AppShell""

**Step 3: Write minimal implementation**

`src/shared/config/navigation.ts`:

```ts
export const PAGES = ["board", "profile", "resumes", "prep", "settings"] as const;
export type PageId = (typeof PAGES)[number];
```

`src/widgets/app-shell/TitleBar.tsx`:

```tsx
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";

export type WindowControls = {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
};

type Props = { controls?: WindowControls };

export function TitleBar({ controls }: Props) {
  const { t } = useTranslation();
  // Resolved on click: getCurrentWindow() needs the Tauri runtime.
  const run = (action: keyof WindowControls) => () => {
    void (controls ?? getCurrentWindow())[action]();
  };
  const buttons = [
    { action: "minimize", label: t("titlebar.minimize"), Icon: Minus },
    { action: "toggleMaximize", label: t("titlebar.maximize"), Icon: Square },
    { action: "close", label: t("titlebar.close"), Icon: X },
  ] as const;

  return (
    <header
      data-tauri-drag-region
      className="flex h-10 shrink-0 items-center justify-between border-b border-noir-border bg-noir-sidebar pl-4"
    >
      <span data-tauri-drag-region className="flex items-center gap-2 text-sm text-noir-muted">
        <span aria-hidden="true" className="size-2 rounded-full bg-noir-amber" />
        {t("app.name")}
      </span>
      <div className="flex h-full">
        {buttons.map(({ action, label, Icon }) => (
          <button
            key={action}
            type="button"
            aria-label={label}
            onClick={run(action)}
            className="grid h-full w-12 place-items-center text-noir-muted hover:bg-white/5 hover:text-noir-text focus-visible:bg-white/10 focus-visible:outline-none"
          >
            <Icon aria-hidden="true" className="size-3.5" />
          </button>
        ))}
      </div>
    </header>
  );
}
```

`src/widgets/app-shell/Sidebar.tsx`:

```tsx
import { FileText, LayoutGrid, NotebookPen, Settings, UserRound, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { PageId } from "@/shared/config/navigation";
import { cn } from "@/shared/lib/utils";

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

export function Sidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-noir-border bg-noir-sidebar">
      <p className="flex items-center gap-2 px-5 py-5 text-lg font-semibold text-noir-amber">
        <span aria-hidden="true" className="size-2.5 rounded-full bg-noir-amber" />
        {t("app.name")}
      </p>
      <nav aria-label={t("nav.label")} className="flex-1 px-3">
        <ul className="flex flex-col gap-1">
          {NAV.map(({ id, Icon, pending }) => (
            <li key={id}>
              <button
                type="button"
                aria-current={active === id ? "page" : undefined}
                onClick={() => onNavigate(id)}
                className={cn(
                  "flex w-full items-center gap-3 rounded-md border-l-2 border-transparent px-3 py-2.5 text-left text-sm text-noir-muted hover:bg-white/5 hover:text-noir-text focus-visible:outline-2 focus-visible:outline-noir-amber",
                  active === id && "border-noir-amber bg-white/5 text-noir-text",
                )}
              >
                <Icon aria-hidden="true" className="size-4" />
                <span className="flex-1">{t(`nav.${id}`)}</span>
                {pending !== undefined && (
                  <span className="rounded-full bg-noir-amber/15 px-2 font-mono text-xs text-noir-amber">
                    <span aria-hidden="true">{pending}</span>
                    <span className="sr-only">{t("nav.pending", { count: pending })}</span>
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      </nav>
      <footer className="flex flex-col gap-1 border-t border-noir-border px-5 py-4 text-xs text-noir-muted">
        <p className="flex items-center gap-2 text-noir-teal">
          <span aria-hidden="true" className="size-2 rounded-full bg-noir-teal" />
          {t("crawler.active")}
        </p>
        <p>{t("crawler.lastRun", CRAWLER_STATUS)}</p>
        <p className="mt-2 font-serif italic">{t("app.signature")} •</p>
      </footer>
    </aside>
  );
}
```

`src/widgets/app-shell/AppShell.tsx`:

```tsx
import { useState, type ReactNode } from "react";
import type { PageId } from "@/shared/config/navigation";
import { Sidebar } from "./Sidebar";
import { TitleBar, type WindowControls } from "./TitleBar";

type Props = {
  renderPage: (page: PageId) => ReactNode;
  controls?: WindowControls;
};

export function AppShell({ renderPage, controls }: Props) {
  const [page, setPage] = useState<PageId>("board");
  return (
    <div className="noir-backdrop flex h-full flex-col">
      <TitleBar controls={controls} />
      <div className="flex min-h-0 flex-1">
        <Sidebar active={page} onNavigate={setPage} />
        <main className="flex min-w-0 flex-1 flex-col">{renderPage(page)}</main>
      </div>
    </div>
  );
}
```

`src/pages/placeholder/PlaceholderPage.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import type { PageId } from "@/shared/config/navigation";

export function PlaceholderPage({ page }: { page: Exclude<PageId, "board"> }) {
  const { t } = useTranslation();
  return (
    <section className="noir-panel m-6 rounded-xl p-8">
      <h1 className="text-xl font-semibold">{t(`nav.${page}`)}</h1>
      <p className="mt-2 text-noir-muted">{t("placeholder.body")}</p>
    </section>
  );
}
```

`src/app/App.tsx` (the board slot is a placeholder until Task 10):

```tsx
import { useTranslation } from "react-i18next";
import { PlaceholderPage } from "@/pages/placeholder/PlaceholderPage";
import { AppShell } from "@/widgets/app-shell/AppShell";

function BoardSlot() {
  const { t } = useTranslation();
  return <h1 className="p-6 text-2xl font-semibold">{t("board.title")}</h1>;
}

export function App() {
  return (
    <AppShell
      renderPage={(page) => (page === "board" ? <BoardSlot /> : <PlaceholderPage page={page} />)}
    />
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (`✓ AppShell drives the window controls`, `✓ App shows a placeholder for pages not built yet`)

Manual: `pnpm tauri dev`, drag the window by the title bar; minimize, maximize and close work.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 9: Job card with salary and outcome

**Requirement:** REQ-007, REQ-008, REQ-009
**Files:**

- Create: `src/entities/job/model/format.ts`
- Create: `src/entities/job/model/stages.ts`
- Create: `src/entities/job/ui/SalaryLine.tsx`
- Create: `src/entities/job/ui/JobCard.tsx`
- Test: `src/entities/job/model/format.test.ts`
- Test: `src/entities/job/ui/JobCard.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/entities/job/model/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { formatThousands, hasNegotiatedSalary, scoreTone } from "./format";

describe("formatThousands", () => {
  it("rounds monthly pay to thousands", () => {
    expect(formatThousands(28_000)).toBe("28");
    expect(formatThousands(21_500)).toBe("22");
  });
});

describe("scoreTone", () => {
  it("splits scores into three tones", () => {
    expect(scoreTone(96)).toBe("high");
    expect(scoreTone(85)).toBe("good");
    expect(scoreTone(70)).toBe("low");
  });
});

describe("hasNegotiatedSalary", () => {
  it("needs at least one regime", () => {
    expect(hasNegotiatedSalary(null)).toBe(false);
    expect(hasNegotiatedSalary({ pj: null, clt: null })).toBe(false);
    expect(hasNegotiatedSalary({ pj: null, clt: 18_000 })).toBe(true);
  });
});
```

`src/entities/job/ui/JobCard.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { JobCard } from "./JobCard";

describe("JobCard", () => {
  it("shows the job essentials", () => {
    render(<JobCard card={makeCard()} onOpen={() => {}} />);
    expect(screen.getByRole("heading", { name: /Staff Engineer/ })).toBeInTheDocument();
    expect(screen.getByText("Lumen Labs")).toBeInTheDocument();
    expect(screen.getByText("Greenhouse")).toBeInTheDocument();
    expect(screen.getByText("AI/LLM")).toBeInTheDocument();
    expect(screen.getByText("Remoto")).toBeInTheDocument();
    // The score pill reads "Score 94" for screen readers (sr-only prefix).
    expect(screen.getByText("Score 94")).toBeInTheDocument();
  });

  it("shows PJ and CLT when both were negotiated", () => {
    render(<JobCard card={makeCard({ stage: "applied", salary: { pj: 34_000, clt: 26_000 } })} onOpen={() => {}} />);
    expect(screen.getByText("PJ")).toBeInTheDocument();
    expect(screen.getByText("R$ 34k")).toBeInTheDocument();
    expect(screen.getByText("CLT")).toBeInTheDocument();
    expect(screen.getByText("R$ 26k")).toBeInTheDocument();
  });

  it("shows only PJ when CLT was not negotiated", () => {
    render(<JobCard card={makeCard({ stage: "applied", salary: { pj: 30_000, clt: null } })} onOpen={() => {}} />);
    expect(screen.getByText("R$ 30k")).toBeInTheDocument();
    expect(screen.queryByText("CLT")).not.toBeInTheDocument();
  });

  it("hides the salary when nothing was negotiated", () => {
    render(<JobCard card={makeCard({ salary: { pj: null, clt: null } })} onOpen={() => {}} />);
    expect(screen.queryByText("PJ")).not.toBeInTheDocument();
    expect(screen.queryByText(/R\$/)).not.toBeInTheDocument();
  });

  it("shows the outcome of a closed card", () => {
    render(<JobCard card={makeCard({ stage: "closed", outcome: "hired" })} onOpen={() => {}} />);
    expect(screen.getByText("Contratado")).toBeInTheDocument();
  });

  it("shows the AI running status", () => {
    render(<JobCard card={makeCard({ aiRunning: "claude" })} onOpen={() => {}} />);
    expect(screen.getByRole("status")).toHaveTextContent("Claude trabalhando…");
  });

  it("opens the details when the title is activated", async () => {
    const onOpen = vi.fn();
    const card = makeCard();
    render(<JobCard card={card} onOpen={onOpen} />);
    await userEvent.click(screen.getByRole("button", { name: "Abrir detalhes de Staff Engineer" }));
    expect(onOpen).toHaveBeenCalledWith(card);
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/entities/job`
Expected: FAIL with "Failed to resolve import "./format"" and "Failed to resolve import "./JobCard""

**Step 3: Write minimal implementation**

`src/entities/job/model/format.ts`:

```ts
import type { Salary } from "@/shared/bindings/Salary";

export function formatThousands(value: number): string {
  return String(Math.round(value / 1000));
}

export type ScoreTone = "high" | "good" | "low";

export function scoreTone(score: number): ScoreTone {
  if (score >= 90) return "high";
  if (score >= 80) return "good";
  return "low";
}

export function hasNegotiatedSalary(salary: Salary | null): salary is Salary {
  return salary !== null && (salary.pj !== null || salary.clt !== null);
}
```

`src/entities/job/model/stages.ts`:

```ts
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import type { Target } from "@/shared/bindings/Target";

/** Board column order; mirrors `Stage::ALL` in src-tauri/src/domain.rs. */
export const STAGES: readonly Stage[] = ["inbox", "triage", "applied", "in_process", "offer", "closed"];
export const OUTCOMES: readonly Outcome[] = ["hired", "rejected", "discarded", "no_response"];
export const TARGETS: readonly Target[] = ["ai_llm", "full_stack", "tech_lead", "data"];

export const isStage = (value: unknown): value is Stage => STAGES.includes(value as Stage);
```

`src/entities/job/ui/SalaryLine.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import type { Salary } from "@/shared/bindings/Salary";
import { formatThousands, hasNegotiatedSalary } from "../model/format";

export function SalaryLine({ salary }: { salary: Salary | null }) {
  const { t } = useTranslation();
  if (!hasNegotiatedSalary(salary)) return null;
  const regimes = [
    { regime: "pj", value: salary.pj },
    { regime: "clt", value: salary.clt },
  ] as const;
  return (
    <p className="flex gap-4 text-sm">
      {regimes.map(({ regime, value }) =>
        value === null ? null : (
          <span key={regime} className="flex gap-1.5">
            <span className="text-noir-muted">{t(`salary.${regime}`)}</span>
            <span className="font-mono">{t("salary.value", { value: formatThousands(value) })}</span>
          </span>
        ),
      )}
    </p>
  );
}
```

`src/entities/job/ui/JobCard.tsx`:

```tsx
import { FileText, ListChecks, Loader2, MessagesSquare } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { cn } from "@/shared/lib/utils";
import { Badge } from "@/shared/ui/badge";
import { scoreTone } from "../model/format";
import { SalaryLine } from "./SalaryLine";

const SCORE_TONE = {
  high: "border-noir-teal/60 text-noir-teal",
  good: "border-noir-amber/60 text-noir-amber",
  low: "border-noir-border text-noir-muted",
} as const;

const OUTCOME_TONE = {
  hired: "bg-noir-teal/15 text-noir-teal",
  rejected: "bg-noir-red/15 text-noir-red",
  discarded: "bg-white/5 text-noir-muted",
  no_response: "bg-white/5 text-noir-muted",
} as const;

const capitalize = (value: string) => value.charAt(0).toUpperCase() + value.slice(1);

type Props = {
  card: BoardCard;
  onOpen: (card: BoardCard) => void;
  /** Slot for card actions (the move menu); sits above the stretched title button. */
  actions?: ReactNode;
};

export function JobCard({ card, onOpen, actions }: Props) {
  const { t } = useTranslation();
  const titleId = `job-${card.id}-title`;
  const artifacts = [
    { key: "cv", done: card.artifacts.cv, Icon: FileText },
    { key: "coverage", done: card.artifacts.coverage, Icon: ListChecks },
    { key: "interviewPrep", done: card.artifacts.interviewPrep, Icon: MessagesSquare },
  ] as const;
  const note = card.nextAction ?? card.lastActivity;

  return (
    <article aria-labelledby={titleId} className="noir-card relative flex flex-col gap-3 rounded-lg p-4">
      <div className="flex items-start gap-3">
        <span
          aria-hidden="true"
          className="grid size-7 shrink-0 place-items-center rounded-md bg-white/5 font-mono text-xs text-noir-muted"
        >
          {card.company.charAt(0)}
        </span>
        <div className="min-w-0 flex-1">
          <h3 id={titleId} className="text-sm font-semibold leading-snug">
            {/* Stretched over the whole card so a click anywhere opens the dialog. */}
            <button
              type="button"
              onClick={() => onOpen(card)}
              aria-label={t("card.open", { title: card.title })}
              className="text-left after:absolute after:inset-0 after:rounded-lg focus-visible:outline-none"
            >
              {card.title}
            </button>
          </h3>
          <p className="truncate text-sm text-noir-muted">{card.company}</p>
        </div>
        {card.unread && (
          <span role="img" aria-label={t("card.unread")} className="mt-1.5 size-2 shrink-0 rounded-full bg-noir-amber" />
        )}
        <span className={cn("rounded-md border px-1.5 py-0.5 font-mono text-xs", SCORE_TONE[scoreTone(card.score)])}>
          <span className="sr-only">{t("card.scoreLabel")} </span>
          {card.score}
        </span>
      </div>

      {card.outcome && (
        <Badge className={cn("w-fit", OUTCOME_TONE[card.outcome])}>{t(`outcome.${card.outcome}`)}</Badge>
      )}

      <div className="flex flex-wrap items-center gap-1.5">
        <Badge variant="secondary">{card.source}</Badge>
        <Badge variant="secondary" className="text-noir-amber">
          {t(`target.${card.target}`)}
        </Badge>
        <span className="text-xs text-noir-muted">{t(`workModel.${card.workModel}`)}</span>
        {card.location && <span className="text-xs text-noir-muted">· {card.location}</span>}
        {card.processStep && (
          <Badge variant="outline" className="border-noir-amber/60 font-mono uppercase text-noir-amber">
            {card.processStep}
          </Badge>
        )}
      </div>

      <SalaryLine salary={card.salary} />

      {card.aiRunning ? (
        <p
          role="status"
          className="flex items-center gap-2 rounded-md border border-noir-amber/40 bg-noir-amber/10 px-2 py-1.5 text-xs text-noir-amber"
        >
          <Loader2 aria-hidden="true" className="size-3.5 animate-spin" />
          {t("card.aiRunning", { cli: capitalize(card.aiRunning) })}
        </p>
      ) : (
        <ul aria-label={t("card.artifacts.label")} className="flex gap-2">
          {artifacts.map(({ key, done, Icon }) => (
            <li key={key}>
              <Icon
                role="img"
                aria-label={`${t(`card.artifacts.${key}`)}: ${t(done ? "card.artifacts.done" : "card.artifacts.missing")}`}
                className={cn("size-4", done ? "text-noir-teal" : "text-noir-muted/40")}
              />
            </li>
          ))}
        </ul>
      )}

      {note && <p className="text-xs text-noir-muted">{note}</p>}
      {actions && <div className="relative z-10 self-end">{actions}</div>}
    </article>
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (3 format tests, 7 JobCard tests)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 10: Board columns and board page

**Requirement:** REQ-007
**Files:**

- Create: `src/widgets/board/model/group.ts`
- Create: `src/widgets/board/BoardColumn.tsx`
- Create: `src/widgets/board/Board.tsx`
- Create: `src/pages/board/BoardPage.tsx`
- Modify: `src/app/App.tsx`
- Test: `src/widgets/board/model/group.test.ts`
- Test: `src/widgets/board/Board.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/widgets/board/model/group.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { groupByStage } from "./group";

describe("groupByStage", () => {
  it("keeps every stage in board order", () => {
    expect(Object.keys(groupByStage([]))).toEqual(["inbox", "triage", "applied", "in_process", "offer", "closed"]);
  });

  it("places each card in its stage", () => {
    const groups = groupByStage([makeCard({ id: "a" }), makeCard({ id: "b", stage: "offer" })]);
    expect(groups.inbox.map((c) => c.id)).toEqual(["a"]);
    expect(groups.offer.map((c) => c.id)).toEqual(["b"]);
  });
});
```

`src/widgets/board/Board.test.tsx`:

```tsx
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { Board } from "./Board";

const cards = [makeCard({ id: "a" }), makeCard({ id: "b" }), makeCard({ id: "c", stage: "offer" })];

describe("Board", () => {
  it("renders the six columns in board order", () => {
    render(<Board cards={cards} onOpen={() => {}} />);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual([
      "Caixa de entrada",
      "Triagem",
      "Candidatado",
      "Em processo",
      "Oferta",
      "Encerradas",
    ]);
  });

  it("announces each column count", () => {
    render(<Board cards={cards} onOpen={() => {}} />);
    const inbox = screen.getByRole("region", { name: "Caixa de entrada" });
    expect(within(inbox).getByText("2 vagas")).toBeInTheDocument();
  });

  it("shows the empty state", () => {
    render(<Board cards={cards} onOpen={() => {}} />);
    const triage = screen.getByRole("region", { name: "Triagem" });
    expect(within(triage).getByText("Nenhuma vaga aqui.")).toBeInTheDocument();
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/widgets/board`
Expected: FAIL with "Failed to resolve import "./group"" and "Failed to resolve import "./Board""

**Step 3: Write minimal implementation**

`src/widgets/board/model/group.ts`:

```ts
import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

export function groupByStage(cards: BoardCard[]): Record<Stage, BoardCard[]> {
  const groups = Object.fromEntries(STAGES.map((stage) => [stage, [] as BoardCard[]])) as Record<
    Stage,
    BoardCard[]
  >;
  for (const card of cards) groups[card.stage].push(card);
  return groups;
}
```

`src/widgets/board/BoardColumn.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { JobCard } from "@/entities/job/ui/JobCard";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";
import { cn } from "@/shared/lib/utils";

type Props = { stage: Stage; cards: BoardCard[]; onOpen: (card: BoardCard) => void };

export function BoardColumn({ stage, cards, onOpen }: Props) {
  const { t } = useTranslation();
  const headingId = `column-${stage}`;
  const isOffer = stage === "offer";
  return (
    <section
      aria-labelledby={headingId}
      className={cn("noir-panel flex min-h-0 flex-col rounded-xl", isOffer && "border-noir-teal/50")}
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
            <li key={card.id}>
              <JobCard card={card} onOpen={onOpen} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
```

`src/widgets/board/Board.tsx`:

```tsx
import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { BoardColumn } from "./BoardColumn";
import { groupByStage } from "./model/group";

type Props = { cards: BoardCard[]; onOpen: (card: BoardCard) => void };

export function Board({ cards, onOpen }: Props) {
  const columns = groupByStage(cards);
  return (
    <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto pb-2">
      {STAGES.map((stage) => (
        <BoardColumn key={stage} stage={stage} cards={columns[stage]} onOpen={onOpen} />
      ))}
    </div>
  );
}
```

`src/pages/board/BoardPage.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { Board } from "@/widgets/board/Board";

type Props = { api?: BoardApi };

export function BoardPage({ api = boardApi }: Props) {
  const { t } = useTranslation();
  const { cards, status } = useBoard(api);
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && <Board cards={cards} onOpen={() => {}} />}
    </div>
  );
}
```

`src/app/App.tsx`:

```tsx
import { BoardPage } from "@/pages/board/BoardPage";
import { PlaceholderPage } from "@/pages/placeholder/PlaceholderPage";
import { AppShell } from "@/widgets/app-shell/AppShell";

export function App() {
  return (
    <AppShell
      renderPage={(page) => (page === "board" ? <BoardPage /> : <PlaceholderPage page={page} />)}
    />
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (2 group tests, 3 Board tests, App test still green)

Manual: `pnpm tauri dev` on the 2560x1080 monitor shows all six columns without horizontal scroll, filled with the Rust mock cards.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 11: Search, filters and metrics strip

**Requirement:** REQ-010
**Files:**

- Create: `src/features/filter-board/model/filter.ts`
- Create: `src/features/filter-board/ui/Toolbar.tsx`
- Create: `src/widgets/board/model/metrics.ts`
- Create: `src/widgets/board/MetricsStrip.tsx`
- Modify: `src/pages/board/BoardPage.tsx`
- Test: `src/features/filter-board/model/filter.test.ts`
- Test: `src/features/filter-board/ui/Toolbar.test.tsx`
- Test: `src/widgets/board/model/metrics.test.ts`
- Test: `src/pages/board/BoardPage.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/features/filter-board/model/filter.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { EMPTY_FILTERS, filterCards } from "./filter";

const v = makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde", source: "Gmail", target: "ai_llm", score: 91 });
const n = makeCard({ id: "n", title: "Engenheiro de Dados", company: "Nébula Data", source: "Lever", target: "data", score: 79 });
const o = makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay", source: "Gupy", target: "tech_lead", score: 82 });
const ids = (cards: { id: string }[]) => cards.map((c) => c.id);

describe("filterCards", () => {
  it("keeps every card with empty filters", () => {
    expect(ids(filterCards([v, n, o], EMPTY_FILTERS))).toEqual(["v", "n", "o"]);
  });

  it("matches company names without accents", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, query: "vertice" }))).toEqual(["v"]);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, query: "SAUDE" }))).toEqual(["v"]);
  });

  it("filters by source and target", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Gupy"], targets: ["tech_lead"] }))).toEqual(["o"]);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Lever", "Gmail"] }))).toEqual(["v", "n"]);
  });

  it("drops cards below the minimum score", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, minScore: 80 }))).toEqual(["v", "o"]);
  });
});
```

`src/widgets/board/model/metrics.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { computeMetrics } from "./metrics";

const now = new Date("2026-11-02T12:00:00Z");
const cards = [
  makeCard({ id: "a", stage: "applied", timeline: [
    { at: "2026-10-30", kind: "applied", label: "" },
    { at: "2026-11-01", kind: "response", label: "" },
  ] }),
  makeCard({ id: "b", stage: "in_process", timeline: [
    { at: "2026-10-20", kind: "applied", label: "" },
    { at: "2026-10-24", kind: "response", label: "" },
  ] }),
  makeCard({ id: "c", stage: "applied", timeline: [{ at: "2026-10-29", kind: "applied", label: "" }] }),
  makeCard({ id: "d", stage: "inbox" }),
];

describe("computeMetrics", () => {
  it("counts applications in the last seven days", () => {
    expect(computeMetrics(cards, now).appliedThisWeek).toBe(2);
  });

  it("computes the response rate", () => {
    expect(computeMetrics(cards, now).responseRate).toBe(67);
  });

  it("counts the cards in process", () => {
    expect(computeMetrics(cards, now).interviewing).toBe(1);
  });

  it("averages the days until a response", () => {
    expect(computeMetrics(cards, now).avgDaysToResponse).toBe(3);
  });

  it("reports no data as null", () => {
    expect(computeMetrics([], now)).toEqual({
      appliedThisWeek: 0,
      responseRate: null,
      interviewing: 0,
      avgDaysToResponse: null,
    });
  });
});
```

`src/features/filter-board/ui/Toolbar.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EMPTY_FILTERS } from "../model/filter";
import { Toolbar } from "./Toolbar";

describe("Toolbar", () => {
  it("reports the search text", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={["Gupy", "Lever"]} onChange={onChange} />);
    await userEvent.type(screen.getByRole("searchbox", { name: "Buscar vaga, empresa…" }), "v");
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, query: "v" });
  });

  it("toggles a source filter", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={["Gupy", "Lever"]} onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Fonte: Todas" }));
    await userEvent.click(await screen.findByRole("menuitemcheckbox", { name: "Gupy" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, sources: ["Gupy"] });
  });
});
```

`src/pages/board/BoardPage.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { fakeApi, makeCard } from "@/test/factories";
import { BoardPage } from "./BoardPage";

const cards = [
  makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde" }),
  makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay" }),
];

describe("BoardPage", () => {
  it("filters the board from the search box", async () => {
    render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { name: /Lead AI Architect/ });
    await userEvent.type(screen.getByRole("searchbox"), "vertice");
    expect(screen.getByRole("heading", { name: /Lead AI Architect/ })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: /Tech Lead/ })).not.toBeInTheDocument();
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/features/filter-board src/widgets/board/model src/pages/board`
Expected: FAIL with "Failed to resolve import "./filter"" (and "./metrics", "./Toolbar")

**Step 3: Write minimal implementation**

`src/features/filter-board/model/filter.ts`:

```ts
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Target } from "@/shared/bindings/Target";

export type BoardFilters = {
  query: string;
  sources: string[];
  targets: Target[];
  minScore: number;
};

export const EMPTY_FILTERS: BoardFilters = { query: "", sources: [], targets: [], minScore: 0 };

/** Lowercase without diacritics, so "vertice" finds "Vértice". */
const fold = (value: string) => value.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase("pt-BR");

export function filterCards(cards: BoardCard[], filters: BoardFilters): BoardCard[] {
  const query = fold(filters.query.trim());
  return cards.filter(
    (card) =>
      (query === "" || fold(card.title).includes(query) || fold(card.company).includes(query)) &&
      (filters.sources.length === 0 || filters.sources.includes(card.source)) &&
      (filters.targets.length === 0 || filters.targets.includes(card.target)) &&
      card.score >= filters.minScore,
  );
}
```

`src/widgets/board/model/metrics.ts`:

```ts
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { EventKind } from "@/shared/bindings/EventKind";

export type BoardMetrics = {
  appliedThisWeek: number;
  /** Percent of applied cards with a response; null when nothing was applied. */
  responseRate: number | null;
  interviewing: number;
  avgDaysToResponse: number | null;
};

const DAY_MS = 86_400_000;

function firstAt(card: BoardCard, kind: EventKind): number | undefined {
  const event = card.timeline.find((e) => e.kind === kind);
  return event ? Date.parse(event.at) : undefined;
}

export function computeMetrics(cards: BoardCard[], now: Date): BoardMetrics {
  const weekAgo = now.getTime() - 7 * DAY_MS;
  const applied = cards.flatMap((card) => {
    const at = firstAt(card, "applied");
    return at === undefined ? [] : [{ card, at }];
  });
  const waits = applied.flatMap(({ card, at }) => {
    const response = firstAt(card, "response");
    return response === undefined ? [] : [(response - at) / DAY_MS];
  });
  return {
    appliedThisWeek: applied.filter(({ at }) => at >= weekAgo).length,
    responseRate: applied.length ? Math.round((waits.length * 100) / applied.length) : null,
    interviewing: cards.filter((card) => card.stage === "in_process").length,
    avgDaysToResponse: waits.length ? Math.round(waits.reduce((a, b) => a + b, 0) / waits.length) : null,
  };
}
```

`src/features/filter-board/ui/Toolbar.tsx`:

```tsx
import { ChevronDown, Mail, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TARGETS } from "@/entities/job/model/stages";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";
import { Input } from "@/shared/ui/input";
import type { BoardFilters } from "../model/filter";

type Props = { filters: BoardFilters; sources: string[]; onChange: (filters: BoardFilters) => void };

const toggle = <T,>(list: T[], value: T): T[] =>
  list.includes(value) ? list.filter((item) => item !== value) : [...list, value];

export function Toolbar({ filters, sources, onChange }: Props) {
  const { t } = useTranslation();
  const summary = (count: number) => (count === 0 ? t("filters.all") : t("filters.selected", { count }));

  return (
    <div role="search" className="flex flex-1 flex-wrap items-center gap-3">
      <div className="relative w-[420px] max-w-full">
        <Search aria-hidden="true" className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-noir-muted" />
        <Input
          type="search"
          aria-label={t("filters.search")}
          placeholder={t("filters.search")}
          value={filters.query}
          onChange={(event) => onChange({ ...filters, query: event.target.value })}
          className="h-9 pl-9"
        />
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="h-9">
            {t("filters.source")}: {summary(filters.sources.length)}
            <ChevronDown aria-hidden="true" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {sources.map((source) => (
            <DropdownMenuCheckboxItem
              key={source}
              checked={filters.sources.includes(source)}
              onCheckedChange={() => onChange({ ...filters, sources: toggle(filters.sources, source) })}
              onSelect={(event) => event.preventDefault()}
            >
              {source}
            </DropdownMenuCheckboxItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="h-9">
            {t("filters.target")}: {summary(filters.targets.length)}
            <ChevronDown aria-hidden="true" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {TARGETS.map((target) => (
            <DropdownMenuCheckboxItem
              key={target}
              checked={filters.targets.includes(target)}
              onCheckedChange={() => onChange({ ...filters, targets: toggle(filters.targets, target) })}
              onSelect={(event) => event.preventDefault()}
            >
              {t(`target.${target}`)}
            </DropdownMenuCheckboxItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <label className="flex h-9 items-center gap-2 rounded-md border border-noir-border px-3 text-sm">
        <span>{t("filters.minScore")}</span>
        <input
          type="range"
          min={0}
          max={100}
          step={5}
          value={filters.minScore}
          onChange={(event) => onChange({ ...filters, minScore: Number(event.target.value) })}
          className="w-24 accent-noir-amber"
        />
        <span className="w-6 font-mono">{filters.minScore}</span>
      </label>

      {/* Gmail sync arrives with the AI CLI runner; disabled until then. */}
      <Button variant="outline" className="ml-auto h-9" disabled title={t("dialog.comingSoon")}>
        <Mail aria-hidden="true" />
        {t("filters.syncGmail")}
      </Button>
    </div>
  );
}
```

`src/widgets/board/MetricsStrip.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import type { BoardMetrics } from "./model/metrics";

export function MetricsStrip({ metrics }: { metrics: BoardMetrics }) {
  const { t } = useTranslation();
  const none = t("metrics.none");
  const items = [
    { label: t("metrics.appliedWeek"), value: String(metrics.appliedThisWeek) },
    { label: t("metrics.responseRate"), value: metrics.responseRate === null ? none : `${metrics.responseRate}%` },
    { label: t("metrics.interviews"), value: String(metrics.interviewing) },
    {
      label: t("metrics.avgResponse"),
      value: metrics.avgDaysToResponse === null ? none : t("metrics.days", { value: metrics.avgDaysToResponse }),
    },
  ];
  return (
    <dl className="flex flex-wrap gap-x-8 gap-y-1 text-sm">
      {items.map(({ label, value }) => (
        <div key={label} className="flex gap-2">
          <dt className="text-noir-muted">{label}</dt>
          <dd className="font-mono">{value}</dd>
        </div>
      ))}
    </dl>
  );
}
```

`src/pages/board/BoardPage.tsx`:

```tsx
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { EMPTY_FILTERS, filterCards, type BoardFilters } from "@/features/filter-board/model/filter";
import { Toolbar } from "@/features/filter-board/ui/Toolbar";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { Board } from "@/widgets/board/Board";
import { MetricsStrip } from "@/widgets/board/MetricsStrip";
import { computeMetrics } from "@/widgets/board/model/metrics";

type Props = { api?: BoardApi; now?: Date };

export function BoardPage({ api = boardApi, now }: Props) {
  const { t } = useTranslation();
  const { cards, status } = useBoard(api);
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_FILTERS);
  const visible = useMemo(() => filterCards(cards, filters), [cards, filters]);
  // Metrics follow the filters, so a source or target can be judged on its own.
  const metrics = useMemo(() => computeMetrics(visible, now ?? new Date()), [visible, now]);
  const sources = useMemo(() => [...new Set(cards.map((card) => card.source))].sort(), [cards]);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <div className="flex flex-wrap items-center gap-4">
        <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
        <Toolbar filters={filters} sources={sources} onChange={setFilters} />
      </div>
      <MetricsStrip metrics={metrics} />
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && <Board cards={visible} onOpen={() => {}} />}
    </div>
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (4 filterCards, 5 computeMetrics, 2 Toolbar, 1 BoardPage tests)

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 12: Move cards (drag-and-drop, "Mover para" menu, outcome dialog)

**Requirement:** REQ-011, REQ-005
**Files:**

- Create: `src/features/move-card/ui/MoveMenu.tsx`
- Create: `src/features/move-card/ui/OutcomeDialog.tsx`
- Modify: `src/widgets/board/Board.tsx`
- Modify: `src/widgets/board/BoardColumn.tsx`
- Test: `src/widgets/board/Board.test.tsx` (replaces the Task 10 test)
- Modify: `src/pages/board/BoardPage.tsx`
- Test: `src/features/move-card/ui/OutcomeDialog.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

`src/features/move-card/ui/OutcomeDialog.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { OutcomeDialog } from "./OutcomeDialog";

describe("OutcomeDialog", () => {
  it("returns the chosen outcome", async () => {
    const onPick = vi.fn();
    render(<OutcomeDialog open onPick={onPick} onCancel={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Recusada" }));
    expect(onPick).toHaveBeenCalledWith("rejected");
  });

  it("cancels without moving", async () => {
    const onPick = vi.fn();
    const onCancel = vi.fn();
    render(<OutcomeDialog open onPick={onPick} onCancel={onCancel} />);
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onPick).not.toHaveBeenCalled();
  });
});
```

`src/widgets/board/Board.test.tsx` (full replacement):

```tsx
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { Board } from "./Board";

const cards = [
  makeCard({ id: "a", title: "Alpha Engineer" }),
  makeCard({ id: "b", title: "Beta Engineer" }),
  makeCard({ id: "c", title: "Gamma Lead", stage: "offer" }),
];
const noop = () => {};

describe("Board", () => {
  it("renders the six columns in board order", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual([
      "Caixa de entrada",
      "Triagem",
      "Candidatado",
      "Em processo",
      "Oferta",
      "Encerradas",
    ]);
  });

  it("announces each column count", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    const inbox = screen.getByRole("region", { name: "Caixa de entrada" });
    expect(within(inbox).getByText("2 vagas")).toBeInTheDocument();
  });

  it("shows the empty state", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    const triage = screen.getByRole("region", { name: "Triagem" });
    expect(within(triage).getByText("Nenhuma vaga aqui.")).toBeInTheDocument();
  });

  it("moves a card through the Mover para menu", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Alpha Engineer" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Triagem" }));
    expect(onMove).toHaveBeenCalledWith("a", "triage", null);
  });

  it("closes a card with the chosen outcome", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Alpha Engineer" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Contratado" }));
    expect(onMove).toHaveBeenCalledWith("a", "closed", "hired");
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/features/move-card src/widgets/board`
Expected: FAIL with "Failed to resolve import "./OutcomeDialog"" and "Unable to find an accessible element with the role "button" and name "Ações de Alpha Engineer""

**Step 3: Write minimal implementation**

`src/features/move-card/ui/MoveMenu.tsx`:

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
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";

type Props = { card: BoardCard; onMove: (stage: Stage, outcome: Outcome | null) => void };

/** Keyboard path for every move that drag-and-drop offers. */
export function MoveMenu({ card, onMove }: Props) {
  const { t } = useTranslation();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="size-7" aria-label={t("card.menu", { title: card.title })}>
          <MoreHorizontal aria-hidden="true" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>{t("move.to")}</DropdownMenuLabel>
        {STAGES.filter((stage) => stage !== "closed" && stage !== card.stage).map((stage) => (
          <DropdownMenuItem key={stage} onSelect={() => onMove(stage, null)}>
            {t(`stage.${stage}`)}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuLabel>{t("move.closeAs")}</DropdownMenuLabel>
        {OUTCOMES.filter((outcome) => outcome !== card.outcome).map((outcome) => (
          <DropdownMenuItem key={outcome} onSelect={() => onMove("closed", outcome)}>
            {t(`outcome.${outcome}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

`src/features/move-card/ui/OutcomeDialog.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { OUTCOMES } from "@/entities/job/model/stages";
import type { Outcome } from "@/shared/bindings/Outcome";
import { Button } from "@/shared/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/shared/ui/dialog";

type Props = { open: boolean; onPick: (outcome: Outcome) => void; onCancel: () => void };

export function OutcomeDialog({ open, onPick, onCancel }: Props) {
  const { t } = useTranslation();
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onCancel()}>
      <DialogContent showCloseButton={false} className="noir-panel">
        <DialogHeader>
          <DialogTitle>{t("move.pickOutcome")}</DialogTitle>
          <DialogDescription className="sr-only">{t("move.closeAs")}</DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-2">
          {OUTCOMES.map((outcome) => (
            <Button key={outcome} variant="outline" onClick={() => onPick(outcome)}>
              {t(`outcome.${outcome}`)}
            </Button>
          ))}
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={onCancel}>
            {t("move.cancel")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

`src/widgets/board/BoardColumn.tsx` (full replacement):

```tsx
import { useDraggable, useDroppable } from "@dnd-kit/core";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { JobCard } from "@/entities/job/ui/JobCard";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";
import { cn } from "@/shared/lib/utils";

function DraggableCard({ card, children }: { card: BoardCard; children: ReactNode }) {
  // Pointer listeners only: the keyboard path is the card's "Mover para" menu.
  const { setNodeRef, listeners, isDragging } = useDraggable({ id: card.id });
  return (
    <li ref={setNodeRef} {...listeners} className={cn("touch-none", isDragging && "opacity-40")}>
      {children}
    </li>
  );
}

type Props = {
  stage: Stage;
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  renderActions: (card: BoardCard) => ReactNode;
};

export function BoardColumn({ stage, cards, onOpen, renderActions }: Props) {
  const { t } = useTranslation();
  const { setNodeRef, isOver } = useDroppable({ id: stage });
  const headingId = `column-${stage}`;
  const isOffer = stage === "offer";
  return (
    <section
      ref={setNodeRef}
      aria-labelledby={headingId}
      className={cn(
        "noir-panel flex min-h-0 flex-col rounded-xl",
        isOffer && "border-noir-teal/50",
        isOver && "border-noir-amber",
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
            <DraggableCard key={card.id} card={card}>
              <JobCard card={card} onOpen={onOpen} actions={renderActions(card)} />
            </DraggableCard>
          ))}
        </ul>
      )}
    </section>
  );
}
```

`src/widgets/board/Board.tsx` (full replacement):

```tsx
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  type Announcements,
  type DragEndEvent,
  type UniqueIdentifier,
} from "@dnd-kit/core";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { STAGES, isStage } from "@/entities/job/model/stages";
import { JobCard } from "@/entities/job/ui/JobCard";
import { MoveMenu } from "@/features/move-card/ui/MoveMenu";
import { OutcomeDialog } from "@/features/move-card/ui/OutcomeDialog";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import { BoardColumn } from "./BoardColumn";
import { groupByStage } from "./model/group";

type Props = {
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  onMove: (id: string, stage: Stage, outcome: Outcome | null) => void;
};

export function Board({ cards, onOpen, onMove }: Props) {
  const { t } = useTranslation();
  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
  const [pendingClose, setPendingClose] = useState<string | null>(null);
  // A 6 px threshold keeps plain clicks opening the dialog.
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 6 } }));
  const columns = groupByStage(cards);
  const find = (id: UniqueIdentifier | null | undefined) => cards.find((card) => card.id === id);
  const title = (id: UniqueIdentifier) => find(id)?.title ?? "";
  const column = (id: UniqueIdentifier) => (isStage(id) ? t(`stage.${id}`) : "");

  const announcements: Announcements = {
    onDragStart: ({ active }) => t("dnd.picked", { title: title(active.id) }),
    onDragOver: ({ active, over }) =>
      over ? t("dnd.over", { title: title(active.id), column: column(over.id) }) : undefined,
    onDragEnd: ({ active, over }) =>
      over
        ? t("dnd.dropped", { title: title(active.id), column: column(over.id) })
        : t("dnd.cancelled", { title: title(active.id) }),
    onDragCancel: ({ active }) => t("dnd.cancelled", { title: title(active.id) }),
  };

  const handleDragEnd = ({ active, over }: DragEndEvent) => {
    setActiveId(null);
    const card = find(active.id);
    if (!card || !over || !isStage(over.id) || over.id === card.stage) return;
    // Closing needs an outcome first; nothing moves until one is picked.
    if (over.id === "closed") setPendingClose(card.id);
    else onMove(card.id, over.id, null);
  };

  const activeCard = find(activeId);

  return (
    <DndContext
      sensors={sensors}
      accessibility={{ announcements }}
      onDragStart={({ active }) => setActiveId(active.id)}
      onDragCancel={() => setActiveId(null)}
      onDragEnd={handleDragEnd}
    >
      <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto pb-2">
        {STAGES.map((stage) => (
          <BoardColumn
            key={stage}
            stage={stage}
            cards={columns[stage]}
            onOpen={onOpen}
            renderActions={(card) => (
              <MoveMenu card={card} onMove={(target, outcome) => onMove(card.id, target, outcome)} />
            )}
          />
        ))}
      </div>
      {/* The overlay escapes the column's scroll clipping while dragging. */}
      <DragOverlay>{activeCard ? <JobCard card={activeCard} onOpen={() => {}} /> : null}</DragOverlay>
      <OutcomeDialog
        open={pendingClose !== null}
        onCancel={() => setPendingClose(null)}
        onPick={(outcome) => {
          if (pendingClose) onMove(pendingClose, "closed", outcome);
          setPendingClose(null);
        }}
      />
    </DndContext>
  );
}
```

`src/pages/board/BoardPage.tsx` (wire moves and the move error; full replacement):

```tsx
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { EMPTY_FILTERS, filterCards, type BoardFilters } from "@/features/filter-board/model/filter";
import { Toolbar } from "@/features/filter-board/ui/Toolbar";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { Board } from "@/widgets/board/Board";
import { MetricsStrip } from "@/widgets/board/MetricsStrip";
import { computeMetrics } from "@/widgets/board/model/metrics";

type Props = { api?: BoardApi; now?: Date };

export function BoardPage({ api = boardApi, now }: Props) {
  const { t } = useTranslation();
  const { cards, status, moveError, move } = useBoard(api);
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_FILTERS);
  const visible = useMemo(() => filterCards(cards, filters), [cards, filters]);
  // Metrics follow the filters, so a source or target can be judged on its own.
  const metrics = useMemo(() => computeMetrics(visible, now ?? new Date()), [visible, now]);
  const sources = useMemo(() => [...new Set(cards.map((card) => card.source))].sort(), [cards]);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <div className="flex flex-wrap items-center gap-4">
        <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
        <Toolbar filters={filters} sources={sources} onChange={setFilters} />
      </div>
      <MetricsStrip metrics={metrics} />
      {moveError && <p role="alert" className="text-sm text-noir-red">{t("board.moveError")}</p>}
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && (
        <Board cards={visible} onOpen={() => {}} onMove={(id, stage, outcome) => void move(id, stage, outcome)} />
      )}
    </div>
  );
}
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (2 OutcomeDialog tests, 5 Board tests, earlier suites green)

Manual: `pnpm tauri dev`; drag a card from Caixa de entrada to Triagem (moves, survives a filter change); drag one to Encerradas (outcome dialog opens; Cancelar leaves it in place; Recusada moves it with the badge); click a card without dragging (nothing breaks; the dialog arrives in Task 13).

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 13: Card details dialog

**Requirement:** REQ-012
**Files:**

- Create: `src/entities/job/ui/JobDialog.tsx`
- Modify: `src/pages/board/BoardPage.tsx`
- Test: `src/entities/job/ui/JobDialog.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: type-safety-reviewer, test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

```tsx
// src/entities/job/ui/JobDialog.test.tsx
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { JobDialog } from "./JobDialog";

const card = makeCard({
  stage: "applied",
  salary: { pj: 28_000, clt: null },
  timeline: [{ at: "2026-10-30", kind: "applied", label: "Candidatura enviada" }],
});

describe("JobDialog", () => {
  it("shows the details and the timeline", () => {
    render(<JobDialog card={card} onOpenChange={() => {}} />);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "Staff Engineer" })).toBeInTheDocument();
    expect(within(dialog).getByText("R$ 28k")).toBeInTheDocument();
    expect(within(dialog).getByText("Candidatura enviada")).toBeInTheDocument();
  });

  it("keeps CV generation disabled until it exists", () => {
    render(<JobDialog card={card} onOpenChange={() => {}} />);
    const button = screen.getByRole("button", { name: "Gerar CV customizado" });
    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Em breve");
  });

  it("closes with Escape", async () => {
    const onOpenChange = vi.fn();
    render(<JobDialog card={card} onOpenChange={onOpenChange} />);
    await userEvent.keyboard("{Escape}");
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/entities/job/ui/JobDialog.test.tsx`
Expected: FAIL with "Failed to resolve import "./JobDialog""

**Step 3: Write minimal implementation**

`src/entities/job/ui/JobDialog.tsx`:

```tsx
import { Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { Button } from "@/shared/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/shared/ui/dialog";
import { hasNegotiatedSalary } from "../model/format";
import { SalaryLine } from "./SalaryLine";

type Props = { card: BoardCard | null; onOpenChange: (open: boolean) => void };

export function JobDialog({ card, onOpenChange }: Props) {
  const { t } = useTranslation();
  return (
    <Dialog open={card !== null} onOpenChange={onOpenChange}>
      {card && (
        <DialogContent showCloseButton={false} className="noir-panel max-w-2xl">
          <DialogHeader>
            <DialogTitle>{card.title}</DialogTitle>
            <DialogDescription>
              {[card.company, card.source, t(`target.${card.target}`), t(`workModel.${card.workModel}`)].join(" · ")}
            </DialogDescription>
          </DialogHeader>

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
              {(["cv", "coverage", "interviewPrep"] as const).map((key) => (
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

          <DialogFooter className="items-center gap-3">
            <span id="job-dialog-cv-hint" className="text-xs text-noir-muted">
              {t("dialog.comingSoon")}
            </span>
            {/* Enabled once the AI CLI runner exists (later plan). */}
            <Button disabled aria-describedby="job-dialog-cv-hint">
              <Sparkles aria-hidden="true" />
              {t("dialog.generateCv")}
            </Button>
            <DialogClose asChild>
              <Button variant="ghost">{t("dialog.close")}</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      )}
    </Dialog>
  );
}
```

`src/pages/board/BoardPage.tsx`: add the open-card state and the dialog. Apply exactly these changes to the Task 12 version:

```tsx
// imports: add
import { JobDialog } from "@/entities/job/ui/JobDialog";

// inside BoardPage, after the `sources` memo:
const [openId, setOpenId] = useState<string | null>(null);
const openCard = cards.find((card) => card.id === openId) ?? null;

// replace the Board line:
<Board
  cards={visible}
  onOpen={(card) => setOpenId(card.id)}
  onMove={(id, stage, outcome) => void move(id, stage, outcome)}
/>

// last child of the root div:
<JobDialog card={openCard} onOpenChange={(open) => !open && setOpenId(null)} />
```

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint`
Expected: PASS (3 JobDialog tests, earlier suites green)

Manual: `pnpm tauri dev`; click a card (dialog opens, focus moves into it); Esc closes and focus returns to the card.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)

---

### Task 14: Accessibility gate on the board page

**Requirement:** REQ-014
**Files:**

- Create: `src/test/axe.ts`
- Test: `src/pages/board/BoardPage.a11y.test.tsx`

**Team:**

- Track: frontend
- Implementer: frontend-engineer (skills: frontend-design, nextjs-fsd-conventions)
- Reviewers: test-reviewer, accessibility-guardrails

**Step 1: Write the failing test**

```tsx
// src/pages/board/BoardPage.a11y.test.tsx
import { render, screen } from "@testing-library/react";
import { describe, it } from "vitest";
import { expectNoAxeViolations } from "@/test/axe";
import { fakeApi, makeCard } from "@/test/factories";
import { BoardPage } from "./BoardPage";

const cards = [
  makeCard({ id: "a", title: "Alpha Engineer", unread: true }),
  makeCard({ id: "b", title: "Beta Lead", stage: "triage", aiRunning: "claude" }),
  makeCard({ id: "c", title: "Gamma Architect", stage: "in_process", processStep: "Técnica", salary: { pj: 34_000, clt: 26_000 } }),
  makeCard({ id: "d", title: "Delta Engineer", stage: "closed", outcome: "rejected", salary: { pj: 25_000, clt: null } }),
];

describe("BoardPage", () => {
  it("has no axe violations", async () => {
    const { container } = render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { level: 2, name: "Caixa de entrada" });
    await expectNoAxeViolations(container);
  });
});
```

**Step 2: Run test to verify it fails**
Run: `pnpm vitest run src/pages/board/BoardPage.a11y.test.tsx`
Expected: FAIL with "Failed to resolve import "@/test/axe""

**Step 3: Write minimal implementation**

`src/test/axe.ts`:

```ts
import axe from "axe-core";
import { expect } from "vitest";

/**
 * WCAG 2.1 A/AA rules that jsdom can evaluate. Color contrast is off because
 * jsdom does not compute styles; it is checked by hand against the Noir tokens.
 */
export async function expectNoAxeViolations(container: Element): Promise<void> {
  const results = await axe.run(container, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] },
    rules: { "color-contrast": { enabled: false } },
  });
  const report = results.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
  expect(report).toEqual([]);
}
```

If the run reports violations, fix them in the component that owns the node (never by disabling a rule), and re-run.

**Step 4: Run test to verify it passes**
Run: `pnpm vitest run --reporter=verbose && pnpm typecheck && pnpm lint && pnpm build && cargo test --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
Expected: PASS (`✓ BoardPage has no axe violations`; full web and Rust suites, lint, build and clippy green)

Manual contrast check: muted text `#8b8b94` on card `#16161a` and amber `#f5a524` on `#0b0b0b` meet 4.5:1.

**Step 5: Commit**
Run: `/git commit`
(Do NOT use raw git commands. The `/git commit` skill applies the project's commit standards automatically.)
