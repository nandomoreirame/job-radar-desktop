import type { Announcements, UniqueIdentifier } from '@dnd-kit/core';
import type { TFunction } from 'i18next';
import { isStage } from '@/entities/job/model/stages';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { Stage } from '@/shared/bindings/Stage';

/**
 * Screen-reader narration for pointer drags. Pure so the strings can be
 * asserted without driving dnd-kit.
 */
export function buildAnnouncements(
  t: TFunction,
  cards: BoardCard[],
  columnOfCard: (id: UniqueIdentifier) => Stage | undefined = () => undefined,
): Announcements {
  const title = (id: UniqueIdentifier) => cards.find(card => card.id === id)?.title ?? '';
  const column = (id: UniqueIdentifier) => {
    const stage = isStage(id) ? id : columnOfCard(id);
    return stage ? t(`stage.${stage}`) : '';
  };
  return {
    onDragStart: ({ active }) => t('dnd.picked', { title: title(active.id) }),
    onDragOver: ({ active, over }) =>
      over ? t('dnd.over', { title: title(active.id), column: column(over.id) }) : undefined,
    onDragEnd: ({ active, over }) =>
      over
        ? t('dnd.dropped', { title: title(active.id), column: column(over.id) })
        : t('dnd.cancelled', { title: title(active.id) }),
    onDragCancel: ({ active }) => t('dnd.cancelled', { title: title(active.id) }),
  };
}
