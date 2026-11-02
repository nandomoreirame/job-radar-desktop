import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { JobCard } from '@/entities/job/ui/JobCard';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { Stage } from '@/shared/bindings/Stage';
import { cn } from '@/shared/lib/utils';
import { KanbanColumn, KanbanItem } from '@/shared/ui/kanban';

type Props = {
  stage: Stage;
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  renderActions: (card: BoardCard) => ReactNode;
};

export function BoardColumn({ stage, cards, onOpen, renderActions }: Props) {
  const { t } = useTranslation();
  const headingId = `column-${stage}`;
  const isOffer = stage === 'offer';
  return (
    <KanbanColumn
      value={stage}
      render={<section />}
      aria-labelledby={headingId}
      className={cn(
        'noir-panel flex max-h-full min-h-0 min-w-[340px] flex-1 basis-0 flex-col rounded-xl data-dragging:border-noir-amber',
        isOffer && 'border-noir-teal/50',
      )}
    >
      <header className="flex items-center gap-2 px-3 py-2.5">
        <h2 id={headingId} className={cn('text-sm font-semibold', isOffer && 'text-noir-teal')}>
          {t(`stage.${stage}`)}
        </h2>
        <span className="rounded bg-white/5 px-1.5 font-mono text-xs text-noir-muted">
          <span aria-hidden="true">{cards.length}</span>
          <span className="sr-only">{t('board.count', { count: cards.length })}</span>
        </span>
      </header>
      {cards.length === 0 ? (
        <p className="px-4 pb-4 text-sm text-noir-muted">{t('board.empty')}</p>
      ) : (
        <ul className="flex flex-col gap-2 overflow-y-auto px-2 pb-2">
          {cards.map(card => (
            <KanbanItem key={card.id} value={card.id} render={<li />}>
              <JobCard card={card} onOpen={onOpen} actions={renderActions(card)} />
            </KanbanItem>
          ))}
        </ul>
      )}
      {stage === 'inbox' && (
        <button
          type="button"
          disabled
          className="mx-2 mb-2 rounded-md border border-dashed border-noir-border px-3 py-2 text-sm text-noir-muted"
        >
          {t('board.addJob')}
        </button>
      )}
    </KanbanColumn>
  );
}
