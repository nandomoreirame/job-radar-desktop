import { useCallback, useEffect, useRef, useState } from 'react';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import { groupByStage } from './group';
import { columnOf, resolveKanbanDrop, type Columns, type KanbanDrop } from './kanban';

export function useKanbanColumns(cards: BoardCard[]) {
  const [columns, setColumnsState] = useState<Columns>(() => groupByStage(cards));
  const columnsRef = useRef(columns);
  const cardsRef = useRef(cards);
  const origin = useRef<{ id: string; stage: NonNullable<ReturnType<typeof columnOf>> } | null>(null);
  const sameColumns = (left: Columns, right: Columns) =>
    Object.keys(left).every(stage => {
      const key = stage as keyof Columns;
      return (
        left[key].map(card => `${card.id}:${card.stage}`).join('|') ===
        right[key].map(card => `${card.id}:${card.stage}`).join('|')
      );
    });
  const setColumns = useCallback((next: Columns) => {
    columnsRef.current = next;
    setColumnsState(next);
  }, []);
  useEffect(() => {
    const nextColumns = groupByStage(cards);
    if (cards === cardsRef.current || origin.current || sameColumns(columnsRef.current, nextColumns)) return;
    cardsRef.current = cards;
    setColumns(nextColumns);
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
