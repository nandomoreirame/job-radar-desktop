import { renderHook, waitFor, act } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { BoardCard } from "@/shared/bindings/BoardCard";
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

  it("ignores a list that answers after unmount", async () => {
    let answer!: (cards: BoardCard[]) => void;
    const api = { ...fakeApi([]), list: () => new Promise<BoardCard[]>((resolve) => (answer = resolve)) };
    const { unmount } = renderHook(() => useBoard(api));
    unmount();
    await act(async () => answer([makeCard()]));
    // Nothing to assert: a state update after unmount would warn.
  });

  it("ignores a failure that arrives after unmount", async () => {
    let fail!: (error: Error) => void;
    const api = {
      ...fakeApi([]),
      list: () => new Promise<BoardCard[]>((_, reject) => (fail = reject)),
    };
    const { unmount } = renderHook(() => useBoard(api));
    unmount();
    await act(async () => fail(new Error("boom")));
  });
});
