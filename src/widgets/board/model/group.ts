import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

const foundAt = (card: BoardCard) => card.timeline.find((event) => event.kind === "found")?.at ?? "";

function byScoreThenNewest(a: BoardCard, b: BoardCard): number {
  if (a.score !== b.score) return (b.score ?? -1) - (a.score ?? -1);
  return foundAt(b).localeCompare(foundAt(a));
}

export function groupByStage(cards: BoardCard[]): Record<Stage, BoardCard[]> {
  const groups = Object.fromEntries(STAGES.map((stage) => [stage, [] as BoardCard[]])) as Record<
    Stage,
    BoardCard[]
  >;
  for (const card of cards) groups[card.stage].push(card);
  groups.inbox.sort(byScoreThenNewest);
  return groups;
}
