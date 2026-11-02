import { afterEach, describe, expect, it, vi } from "vitest";
import { VISUAL_MODE_KEY, applyVisualMode, readStoredVisualMode, resolveVisualMode } from "./visual-mode";

afterEach(() => localStorage.clear());

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

describe("readStoredVisualMode", () => {
  it("returns the stored choice", () => {
    localStorage.setItem(VISUAL_MODE_KEY, "flat");
    expect(readStoredVisualMode()).toBe("flat");
  });

  it("returns null when storage throws", () => {
    const spy = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    expect(readStoredVisualMode()).toBeNull();
    spy.mockRestore();
  });
});
