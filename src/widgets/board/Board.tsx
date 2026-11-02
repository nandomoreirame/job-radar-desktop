import { GripVertical } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { STAGES } from '@/entities/job/model/stages';
import { JobCard } from '@/entities/job/ui/JobCard';
import { MoveMenu } from '@/features/move-card/ui/MoveMenu';
import { OutcomeDialog } from '@/features/move-card/ui/OutcomeDialog';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { Outcome } from '@/shared/bindings/Outcome';
import type { Stage } from '@/shared/bindings/Stage';
import { Kanban, KanbanBoard, KanbanItemHandle, KanbanOverlay } from '@/shared/ui/kanban';
import { BoardColumn } from './BoardColumn';
import { buildAnnouncements } from './model/announcements';
import { columnOf, type Columns } from './model/kanban';
import { useKanbanColumns } from './model/use-kanban-columns';

type Props = {
  cards: BoardCard[];
  onOpen: (card: BoardCard) => void;
  onMove: (id: string, stage: Stage, outcome: Outcome | null) => void;
};

export function Board({ cards, onOpen, onMove }: Props) {
  const { t } = useTranslation();
  const { columns, setColumns, begin, finish, restore } = useKanbanColumns(cards);
  const [pendingClose, setPendingClose] = useState<string | null>(null);
  const announcements = buildAnnouncements(t, cards, id => columnOf(columns, id));
  const handleDragEnd = () => {
    const drop = finish();
    if (!drop) return;
    if (drop.kind === 'close') setPendingClose(drop.id);
    else onMove(drop.id, drop.stage, null);
  };
  return (
    <>
      <Kanban
        value={columns}
        onValueChange={next => setColumns(next as Columns)}
        getItemValue={card => card.id}
        accessibility={{ announcements, screenReaderInstructions: { draggable: t('dnd.instructions') } }}
        onDragStart={({ active }) => begin(String(active.id))}
        onDragEnd={handleDragEnd}
        onDragCancel={() => {
          finish();
          restore();
        }}
      >
        <KanbanBoard className="-mx-6 min-h-0 flex-1 items-start gap-3 overflow-x-auto pb-6 pl-6">
          {STAGES.map(stage => (
            <BoardColumn
              key={stage}
              stage={stage}
              cards={columns[stage]}
              onOpen={onOpen}
              renderActions={card => (
                <>
                  <KanbanItemHandle
                    aria-label={t('card.drag', { title: card.title })}
                    className="grid size-7 place-items-center rounded-md text-noir-muted hover:bg-white/5 focus-visible:outline-2 focus-visible:outline-noir-amber"
                  >
                    <GripVertical aria-hidden="true" className="size-4" />
                  </KanbanItemHandle>
                  <MoveMenu card={card} onMove={(target, outcome) => onMove(card.id, target, outcome)} />
                </>
              )}
            />
          ))}
          {/* Trailing space inside the scroller: WebKitGTK ignores end padding of an overflow
              container, so the gap before the right edge is a spacer instead of pr-6. */}
          <div aria-hidden="true" className="w-3 shrink-0" />
        </KanbanBoard>
        <KanbanOverlay>
          {({ value }) => {
            const card = cards.find(item => item.id === value);
            return card ? <JobCard card={card} onOpen={() => {}} /> : null;
          }}
        </KanbanOverlay>
      </Kanban>
      <OutcomeDialog
        open={pendingClose !== null}
        onCancel={() => {
          setPendingClose(null);
          restore();
        }}
        onPick={outcome => {
          if (pendingClose) onMove(pendingClose, 'closed', outcome);
          setPendingClose(null);
        }}
      />
    </>
  );
}
