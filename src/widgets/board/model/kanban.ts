import type { UniqueIdentifier } from '@dnd-kit/core';
import { STAGES } from '@/entities/job/model/stages';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { Stage } from '@/shared/bindings/Stage';

export type Columns = Record<Stage, BoardCard[]>;
export type KanbanDrop = { id: string; kind: 'move'; stage: Stage } | { id: string; kind: 'close' } | null;
export function columnOf(columns: Columns, id: UniqueIdentifier): Stage | undefined {
  return STAGES.find(stage => columns[stage].some(card => card.id === id));
}
export function resolveKanbanDrop(id: string, origin: Stage, final: Stage | undefined): KanbanDrop {
  if (!final || final === origin) return null;
  return final === 'closed' ? { id, kind: 'close' } : { id, kind: 'move', stage: final };
}
