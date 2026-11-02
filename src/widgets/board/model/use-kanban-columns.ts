import { useCallback, useEffect, useRef, useState } from "react";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import { groupByStage } from "./group";
import { columnOf, resolveKanbanDrop, type Columns, type KanbanDrop } from "./kanban";

export function useKanbanColumns(cards: BoardCard[]) {
  const [columns, setColumnsState] = useState<Columns>(() => groupByStage(cards));
  const columnsRef = useRef(columns);
  const cardsRef = useRef(cards);
  const cardsSignature = useRef(JSON.stringify(cards));
  const origin = useRef<{ id: string; stage: NonNullable<ReturnType<typeof columnOf>> } | null>(null);
  const setColumns = useCallback((next: Columns) => { columnsRef.current = next; setColumnsState(next); }, []);
  useEffect(() => {
    const nextSignature = JSON.stringify(cards);
    if (nextSignature === cardsSignature.current) return;
    cardsSignature.current = nextSignature;
    cardsRef.current = cards;
    setColumns(groupByStage(cards));
  }, [cards, setColumns]);
  const begin = useCallback((id: string) => { const stage = columnOf(columnsRef.current, id); origin.current = stage ? { id, stage } : null; }, []);
  const finish = useCallback((): KanbanDrop => { const start = origin.current; origin.current = null; if (!start) return null; return resolveKanbanDrop(start.id, start.stage, columnOf(columnsRef.current, start.id)); }, []);
  const restore = useCallback(() => setColumns(groupByStage(cardsRef.current)), [setColumns]);
  return { columns, setColumns, begin, finish, restore };
}
