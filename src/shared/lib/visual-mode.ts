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
