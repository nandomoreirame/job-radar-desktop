import { STAGES } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { BoardColumn } from "./BoardColumn";
import { groupByStage } from "./model/group";

type Props = { cards: BoardCard[]; onOpen: (card: BoardCard) => void };

export function Board({ cards, onOpen }: Props) {
  const columns = groupByStage(cards);
  return (
    <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto pb-2">
      {STAGES.map((stage) => (
        <BoardColumn key={stage} stage={stage} cards={columns[stage]} onOpen={onOpen} />
      ))}
    </div>
  );
}
