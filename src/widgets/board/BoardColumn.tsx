import { useDraggable, useDroppable } from "@dnd-kit/core";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { JobCard } from "@/entities/job/ui/JobCard";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";
import { cn } from "@/shared/lib/utils";

function DraggableCard({ card, children }: { card: BoardCard; children: ReactNode }) {
  // Pointer listeners only: the keyboard path is the card's "Mover para" menu.
  const { setNodeRef, listeners, isDragging } = useDraggable({ id: card.id });
  return (
    <li ref={setNodeRef} {...listeners} className={cn("touch-none", isDragging && "opacity-40")}>
      {children}
    </li>
  );
}

type Props = {
  stage: Stage;
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  renderActions: (card: BoardCard) => ReactNode;
};

export function BoardColumn({ stage, cards, onOpen, renderActions }: Props) {
  const { t } = useTranslation();
  const { setNodeRef, isOver } = useDroppable({ id: stage });
  const headingId = `column-${stage}`;
  const isOffer = stage === "offer";
  return (
    <section
      ref={setNodeRef}
      aria-labelledby={headingId}
      className={cn(
        "noir-panel flex min-h-0 flex-col rounded-xl",
        isOffer && "border-noir-teal/50",
        isOver && "border-noir-amber",
      )}
    >
      <header className="flex items-center gap-2 px-4 py-3">
        <h2 id={headingId} className={cn("text-sm font-semibold", isOffer && "text-noir-teal")}>
          {t(`stage.${stage}`)}
        </h2>
        <span className="rounded bg-white/5 px-1.5 font-mono text-xs text-noir-muted">
          <span aria-hidden="true">{cards.length}</span>
          <span className="sr-only">{t("board.count", { count: cards.length })}</span>
        </span>
      </header>
      {cards.length === 0 ? (
        <p className="px-4 pb-4 text-sm text-noir-muted">{t("board.empty")}</p>
      ) : (
        <ul className="flex flex-col gap-3 overflow-y-auto px-3 pb-3">
          {cards.map((card) => (
            <DraggableCard key={card.id} card={card}>
              <JobCard card={card} onOpen={onOpen} actions={renderActions(card)} />
            </DraggableCard>
          ))}
        </ul>
      )}
    </section>
  );
}
