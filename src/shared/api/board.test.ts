import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { boardApi } from "./board";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockInvoke = vi.mocked(invoke);

describe("boardApi", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("lists the board through the Rust command", async () => {
    mockInvoke.mockResolvedValueOnce([makeCard()]);
    await expect(boardApi.list()).resolves.toEqual([makeCard()]);
    expect(mockInvoke).toHaveBeenCalledWith("list_board");
  });

  it("sends the move through the Rust command", async () => {
    const moved = makeCard({ stage: "triage" });
    mockInvoke.mockResolvedValueOnce(moved);
    await expect(boardApi.move("c01", "triage", null)).resolves.toEqual(moved);
    expect(mockInvoke).toHaveBeenCalledWith("move_card", { id: "c01", stage: "triage", outcome: null });
  });
});
