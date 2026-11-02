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
