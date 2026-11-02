import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
import { DiscardedPage } from './DiscardedPage';
const job: DiscardedJob = {
  id: 'aaaaaaaaaaaaaaaa',
  title: 'Data Engineer',
  company: 'Beta',
  source: 'greenhouse',
  location: 'Remoto',
  score: null,
  reason: 'abaixo do corte de score',
  discardedAt: '2026-11-02',
  jobUrl: 'https://a.example/1',
  description: 'Python e AWS',
};
function api(jobs: DiscardedJob[]): DiscardedApi {
  let state = [...jobs];
  return {
    list: async () => state,
    restore: vi.fn(async id => {
      state = state.filter(j => j.id !== id);
      return {} as never;
    }),
    remove: vi.fn(async id => {
      state = state.filter(j => j.id !== id);
    }),
    openLink: vi.fn(async () => {}),
  };
}
describe('DiscardedPage', () => {
  it('lists and searches discarded jobs', async () => {
    const other = { ...job, id: 'bbbbbbbbbbbbbbbb', title: 'Tech Lead', company: 'Coração Labs', description: '' };
    render(<DiscardedPage api={api([job, other])} />);
    expect(await screen.findByRole('heading', { name: 'Data Engineer' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox'), 'coracao');
    expect(screen.queryByRole('heading', { name: 'Data Engineer' })).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Tech Lead' })).toBeInTheDocument();
  });
  it('restores a job', async () => {
    const a = api([job]);
    render(<DiscardedPage api={a} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Mover para a Caixa de entrada' }));
    expect(a.restore).toHaveBeenCalledWith(job.id);
  });
  it('deletes a job only after confirming', async () => {
    const a = api([job]);
    render(<DiscardedPage api={a} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Excluir' }));
    expect(a.remove).not.toHaveBeenCalled();
    await userEvent.click(
      within(await screen.findByRole('dialog')).getByRole('button', { name: 'Excluir definitivamente' }),
    );
    expect(a.remove).toHaveBeenCalledWith(job.id);
  });
  it('has no axe violations', async () => {
    render(<DiscardedPage api={api([job])} />);
    expect(await screen.findByRole('heading', { name: 'Data Engineer' })).toBeInTheDocument();
  });
});
