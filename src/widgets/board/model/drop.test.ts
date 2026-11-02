import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { resolveDrop } from "./drop";

const card = makeCard({ id: "a", stage: "inbox" });

describe("resolveDrop", () => {
  it("moves to another open stage", () => {
    expect(resolveDrop(card, "triage")).toEqual({ kind: "move", stage: "triage" });
  });

  it("asks for an outcome before closing", () => {
    expect(resolveDrop(card, "closed")).toEqual({ kind: "close" });
  });

  it("ignores a drop on the card's own column", () => {
    expect(resolveDrop(card, "inbox")).toBeNull();
  });

  it("ignores unknown columns and missing data", () => {
    expect(resolveDrop(card, "bogus")).toBeNull();
    expect(resolveDrop(undefined, "triage")).toBeNull();
    expect(resolveDrop(card, undefined)).toBeNull();
  });
});
