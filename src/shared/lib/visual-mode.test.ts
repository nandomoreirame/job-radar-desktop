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
