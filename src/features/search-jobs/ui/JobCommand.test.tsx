import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { makeCard } from '@/test/factories';
import { JobCommand } from './JobCommand';
const cards = [
  makeCard({ id: 'v', title: 'Lead AI Architect', company: 'Vértice Saúde' }),
  makeCard({ id: 'o', title: 'Tech Lead', company: 'Orbita Pay' }),
];
describe('JobCommand', () => {
  it('lists every job when empty', () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    expect(screen.getAllByRole('option')).toHaveLength(2);
  });
  it('finds a job without accents', async () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    await userEvent.type(screen.getByRole('combobox'), 'vertice');
    expect(screen.getAllByRole('option')).toHaveLength(1);
    expect(screen.getByRole('option')).toHaveTextContent('Lead AI Architect');
  });
  it('does not search by internal id', async () => {
    const internal = [makeCard({ id: 'c01', title: 'Visible Job', company: 'Acme' })];
    render(<JobCommand cards={internal} open onOpenChange={() => {}} onPick={() => {}} />);
    await userEvent.type(screen.getByRole('combobox'), 'c0');
    expect(screen.queryAllByRole('option')).toHaveLength(0);
  });
  it('keeps two identical jobs distinct', async () => {
    const twins = [
      makeCard({ id: 't1', title: 'Twin Role', company: 'Same Co' }),
      makeCard({ id: 't2', title: 'Twin Role', company: 'Same Co' }),
    ];
    const onPick = vi.fn();
    render(<JobCommand cards={twins} open onOpenChange={() => {}} onPick={onPick} />);
    await userEvent.type(screen.getByRole('combobox'), 'twin');
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(2);
    await userEvent.click(options[1]);
    expect(onPick).toHaveBeenCalledWith(twins[1]);
  });
  it('shows the empty state', async () => {
    render(<JobCommand cards={cards} open onOpenChange={() => {}} onPick={() => {}} />);
    await userEvent.type(screen.getByRole('combobox'), 'zzz');
    expect(screen.getByText('Nenhuma vaga encontrada.')).toBeInTheDocument();
  });
  it('picks a job and closes', async () => {
    const onPick = vi.fn();
    const onOpenChange = vi.fn();
    render(<JobCommand cards={cards} open onOpenChange={onOpenChange} onPick={onPick} />);
    await userEvent.click(screen.getByRole('option', { name: /Tech Lead/ }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onPick).toHaveBeenCalledWith(cards[1]);
  });
});
