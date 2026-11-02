import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

export function groupByStage(cards: BoardCard[]): Record<Stage, BoardCard[]> {
  const groups = Object.fromEntries(STAGES.map((stage) => [stage, [] as BoardCard[]])) as Record<
    Stage,
    BoardCard[]
  >;
  for (const card of cards) groups[card.stage].push(card);
  return groups;
}
