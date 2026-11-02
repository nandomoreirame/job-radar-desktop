import { fireEvent, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useHotkey } from "./use-hotkey";
describe("useHotkey", () => { it("fires on Ctrl or Cmd plus the key", () => { const handler = vi.fn(); renderHook(() => useHotkey("k", handler)); fireEvent.keyDown(window, { key: "k", ctrlKey: true }); fireEvent.keyDown(window, { key: "K", metaKey: true }); expect(handler).toHaveBeenCalledTimes(2); }); it("ignores the bare key", () => { const handler = vi.fn(); renderHook(() => useHotkey("k", handler)); fireEvent.keyDown(window, { key: "k" }); expect(handler).not.toHaveBeenCalled(); }); });
