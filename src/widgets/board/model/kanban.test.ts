import { describe, expect, it } from 'vitest';
import { makeCard } from '@/test/factories';
import { columnOf, resolveKanbanDrop } from './kanban';
import { groupByStage } from './group';

const columns = groupByStage([makeCard({ id: 'a' }), makeCard({ id: 'b', stage: 'offer' })]);

describe('columnOf', () => {
  it('finds the stage holding a card', () => expect(columnOf(columns, 'b')).toBe('offer'));
  it('returns undefined for an unknown id', () => expect(columnOf(columns, 'zzz')).toBeUndefined());
});

describe('resolveKanbanDrop', () => {
  it('moves to another open stage', () =>
    expect(resolveKanbanDrop('a', 'inbox', 'triage')).toEqual({ id: 'a', kind: 'move', stage: 'triage' }));
  it('asks for an outcome before closing', () =>
    expect(resolveKanbanDrop('a', 'inbox', 'closed')).toEqual({ id: 'a', kind: 'close' }));
  it('ignores the same column and a lost card', () => {
    expect(resolveKanbanDrop('a', 'inbox', 'inbox')).toBeNull();
    expect(resolveKanbanDrop('a', 'inbox', undefined)).toBeNull();
  });
});
