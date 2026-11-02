import { useTranslation } from "react-i18next";
import { JobCard } from "@/entities/job/ui/JobCard";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";
import { cn } from "@/shared/lib/utils";

type Props = { stage: Stage; cards: BoardCard[]; onOpen: (card: BoardCard) => void };

export function BoardColumn({ stage, cards, onOpen }: Props) {
  const { t } = useTranslation();
  const headingId = `column-${stage}`;
  const isOffer = stage === "offer";
  return (
    <section
      aria-labelledby={headingId}
      className={cn("noir-panel flex min-h-0 flex-col rounded-xl", isOffer && "border-noir-teal/50")}
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
            <li key={card.id}>
              <JobCard card={card} onOpen={onOpen} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
