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

describe('groupByStage', () => {
  it('sorts the inbox by score then newest', () => {
    const found = (at: string) => [{ at, kind: 'found' as const, label: 'x' }];
    const cards = [makeCard({ id: 'none', score: null, timeline: found('2026-10-30') }), makeCard({ id: 'old', score: 80, timeline: found('2026-09-01') }), makeCard({ id: 'new', score: 80, timeline: found('2026-10-01') }), makeCard({ id: 'top', score: 90, timeline: found('2026-08-01') })];
    expect(groupByStage(cards).inbox.map(c => c.id)).toEqual(['top', 'new', 'old', 'none']);
  });
  it('keeps store order in other columns', () => {
    const cards = [makeCard({ id: 'a', stage: 'triage', score: 10 }), makeCard({ id: 'b', stage: 'triage', score: 90 })];
    expect(groupByStage(cards).triage.map(c => c.id)).toEqual(['a', 'b']);
  });
});
