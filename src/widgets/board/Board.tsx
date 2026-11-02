import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
  type UniqueIdentifier,
} from "@dnd-kit/core";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { STAGES } from "@/entities/job/model/stages";
import { JobCard } from "@/entities/job/ui/JobCard";
import { MoveMenu } from "@/features/move-card/ui/MoveMenu";
import { OutcomeDialog } from "@/features/move-card/ui/OutcomeDialog";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import { BoardColumn } from "./BoardColumn";
import { buildAnnouncements } from "./model/announcements";
import { resolveDrop } from "./model/drop";
import { groupByStage } from "./model/group";

type Props = {
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  onMove: (id: string, stage: Stage, outcome: Outcome | null) => void;
};

export function Board({ cards, onOpen, onMove }: Props) {
  const { t } = useTranslation();
  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
  const [pendingClose, setPendingClose] = useState<string | null>(null);
  // A 6 px threshold keeps plain clicks opening the dialog.
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 6 } }));
  const columns = groupByStage(cards);
  const find = (id: UniqueIdentifier | null | undefined) => cards.find((card) => card.id === id);
  const announcements = buildAnnouncements(t, cards);

  const handleDragEnd = ({ active, over }: DragEndEvent) => {
    setActiveId(null);
    const card = find(active.id);
    const drop = resolveDrop(card, over?.id);
    if (!card || !drop) return;
    // Closing needs an outcome first; nothing moves until one is picked.
    if (drop.kind === "close") setPendingClose(card.id);
    else onMove(card.id, drop.stage, null);
  };

  const activeCard = find(activeId);

  return (
    <DndContext
      sensors={sensors}
      accessibility={{ announcements }}
      onDragStart={({ active }) => setActiveId(active.id)}
      onDragCancel={() => setActiveId(null)}
      onDragEnd={handleDragEnd}
    >
      <div className="grid min-h-0 flex-1 auto-cols-[minmax(300px,1fr)] grid-flow-col gap-4 overflow-x-auto pb-2">
        {STAGES.map((stage) => (
          <BoardColumn
            key={stage}
            stage={stage}
            cards={columns[stage]}
            onOpen={onOpen}
            renderActions={(card) => (
              <MoveMenu card={card} onMove={(target, outcome) => onMove(card.id, target, outcome)} />
            )}
          />
        ))}
      </div>
      {/* The overlay escapes the column's scroll clipping while dragging. */}
      <DragOverlay>{activeCard ? <JobCard card={activeCard} onOpen={() => {}} /> : null}</DragOverlay>
      <OutcomeDialog
        open={pendingClose !== null}
        onCancel={() => setPendingClose(null)}
        onPick={(outcome) => {
          if (pendingClose) onMove(pendingClose, "closed", outcome);
          setPendingClose(null);
        }}
      />
    </DndContext>
  );
}
