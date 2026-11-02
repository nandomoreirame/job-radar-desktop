import { useCallback, useEffect, useRef, useState } from 'react';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import { groupByStage } from './group';
import { columnOf, resolveKanbanDrop, type Columns, type KanbanDrop } from './kanban';

export function useKanbanColumns(cards: BoardCard[]) {
  const [columns, setColumnsState] = useState<Columns>(() => groupByStage(cards));
  const columnsRef = useRef(columns);
  const cardsRef = useRef(cards);
  const origin = useRef<{ id: string; stage: NonNullable<ReturnType<typeof columnOf>> } | null>(null);
  const setColumns = useCallback((next: Columns) => {
    columnsRef.current = next;
    setColumnsState(next);
  }, []);
  // Every new `cards` array is store truth (a failed move hands out an equal
  // copy on purpose), so resync on identity. Mid-drag, keep Kanban's local
  // columns; `restore` and the next store update bring the latest data back.
  useEffect(() => {
    if (cards === cardsRef.current) return;
    cardsRef.current = cards;
    if (!origin.current) setColumns(groupByStage(cards));
  }, [cards, setColumns]);
  const begin = useCallback((id: string) => {
    const stage = columnOf(columnsRef.current, id);
    origin.current = stage ? { id, stage } : null;
  }, []);
  const finish = useCallback((): KanbanDrop => {
    const start = origin.current;
    origin.current = null;
    if (!start) return null;
    return resolveKanbanDrop(start.id, start.stage, columnOf(columnsRef.current, start.id));
  }, []);
  const restore = useCallback(() => setColumns(groupByStage(cardsRef.current)), [setColumns]);
  return { columns, setColumns, begin, finish, restore };
}
