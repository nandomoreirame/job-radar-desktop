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
