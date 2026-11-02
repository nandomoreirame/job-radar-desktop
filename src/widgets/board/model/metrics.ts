import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { EventKind } from "@/shared/bindings/EventKind";

export type BoardMetrics = {
  appliedThisWeek: number;
  /** Percent of applied cards with a response; null when nothing was applied. */
  responseRate: number | null;
  interviewing: number;
  avgDaysToResponse: number | null;
};

const DAY_MS = 86_400_000;

function firstAt(card: BoardCard, kind: EventKind): number | undefined {
  const event = card.timeline.find((e) => e.kind === kind);
  return event ? Date.parse(event.at) : undefined;
}

export function computeMetrics(cards: BoardCard[], now: Date): BoardMetrics {
  const weekAgo = now.getTime() - 7 * DAY_MS;
  const applied = cards.flatMap((card) => {
    const at = firstAt(card, "applied");
    return at === undefined ? [] : [{ card, at }];
  });
  const waits = applied.flatMap(({ card, at }) => {
    const response = firstAt(card, "response");
    return response === undefined ? [] : [(response - at) / DAY_MS];
  });
  return {
    appliedThisWeek: applied.filter(({ at }) => at >= weekAgo).length,
    responseRate: applied.length ? Math.round((waits.length * 100) / applied.length) : null,
    interviewing: cards.filter((card) => card.stage === "in_process").length,
    avgDaysToResponse: waits.length ? Math.round(waits.reduce((a, b) => a + b, 0) / waits.length) : null,
  };
}
