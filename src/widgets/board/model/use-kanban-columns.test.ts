import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import { makeCard } from '@/test/factories';
import { useKanbanColumns } from './use-kanban-columns';

const a = makeCard({ id: 'a' });
const b = makeCard({ id: 'b' });
const ids = (list: BoardCard[]) => list.map(card => card.id);
function dragTo(result: { current: ReturnType<typeof useKanbanColumns> }, stage: 'triage' | 'closed') {
  const { columns } = result.current;
  act(() => result.current.setColumns({ ...columns, inbox: columns.inbox.filter(c => c.id !== 'a'), [stage]: [a] }));
}

describe('useKanbanColumns', () => {
  it('starts from the store cards', () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    expect(ids(result.current.columns.inbox)).toEqual(['a', 'b']);
  });
  it('resolves a cross-column drag into a move', () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin('a'));
    dragTo(result, 'triage');
    expect(result.current.finish()).toEqual({ id: 'a', kind: 'move', stage: 'triage' });
  });
  it('asks for an outcome when dropped on Encerradas', () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin('a'));
    dragTo(result, 'closed');
    expect(result.current.finish()).toEqual({ id: 'a', kind: 'close' });
  });
  it('ignores a reorder inside the column', () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin('a'));
    act(() => result.current.setColumns({ ...result.current.columns, inbox: [b, a] }));
    expect(result.current.finish()).toBeNull();
  });
  it('restores the store columns', () => {
    const { result } = renderHook(() => useKanbanColumns([a, b]));
    act(() => result.current.begin('a'));
    dragTo(result, 'closed');
    act(() => result.current.restore());
    expect(ids(result.current.columns.inbox)).toEqual(['a', 'b']);
    expect(result.current.columns.closed).toEqual([]);
  });
  it('resyncs after the store returns an equal new list', () => {
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [a, b] },
    });
    act(() => result.current.begin('a'));
    dragTo(result, 'triage');
    act(() => result.current.finish());
    rerender({ cards: [a, b] });
    expect(ids(result.current.columns.inbox)).toEqual(['a', 'b']);
  });
  it('resyncs when the store cards change', () => {
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [a, b] },
    });
    rerender({ cards: [a, { ...b, stage: 'offer' as const }] });
    expect(ids(result.current.columns.offer)).toEqual(['b']);
  });
  it('shows new card data when the card stays in its column', () => {
    const hired = makeCard({ id: 'h', stage: 'closed', outcome: 'hired' });
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [hired] },
    });
    rerender({ cards: [{ ...hired, outcome: 'rejected' as const }] });
    expect(result.current.columns.closed[0].outcome).toBe('rejected');
  });
  it('restores the latest store data after a cancelled drag', () => {
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [a, b] },
    });
    rerender({ cards: [{ ...a, title: 'Renamed' }, b] });
    act(() => result.current.begin('a'));
    dragTo(result, 'triage');
    act(() => result.current.restore());
    expect(result.current.columns.inbox[0].title).toBe('Renamed');
  });
  it('keeps the dragged columns while a drag is in progress', () => {
    const { result, rerender } = renderHook(({ cards }) => useKanbanColumns(cards), {
      initialProps: { cards: [a, b] },
    });
    act(() => result.current.begin('a'));
    dragTo(result, 'triage');
    rerender({ cards: [a, b] });
    expect(ids(result.current.columns.triage)).toEqual(['a']);
  });
  it('finishes nothing without a drag in progress', () => {
    const { result } = renderHook(() => useKanbanColumns([a]));
    expect(result.current.finish()).toBeNull();
  });
});
