import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
import { expectNoAxeViolations } from '@/test/axe';
import { makeCard } from '@/test/factories';
import { DiscardedPage } from './DiscardedPage';

const job = (overrides: Partial<DiscardedJob> = {}): DiscardedJob => ({
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
  ...overrides,
});

function fakeDiscarded(jobs: DiscardedJob[]): DiscardedApi {
  let state = [...jobs];
  return {
    list: vi.fn(async () => state),
    restore: vi.fn(async (id: string) => {
      state = state.filter(j => j.id !== id);
      return makeCard({ id });
    }),
    remove: vi.fn(async (id: string) => {
      state = state.filter(j => j.id !== id);
    }),
    openLink: vi.fn(async () => {}),
  };
}

const other = job({ id: 'bbbbbbbbbbbbbbbb', title: 'Tech Lead', company: 'Coração Labs', description: '' });

describe('DiscardedPage', () => {
  it('lists and searches discarded jobs', async () => {
    render(<DiscardedPage api={fakeDiscarded([job(), other])} />);
    expect(await screen.findByRole('heading', { name: 'Data Engineer' })).toBeInTheDocument();
    expect(screen.getAllByText('abaixo do corte de score', { exact: false })).toHaveLength(2);
    expect(screen.getByText('Python e AWS')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Buscar descartadas' }), 'coracao');
    expect(screen.queryByRole('heading', { name: 'Data Engineer' })).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Tech Lead' })).toBeInTheDocument();
  });

  it('restores a job', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Mover para a Caixa de entrada' }));
    expect(api.restore).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
    await waitFor(() => expect(screen.queryByRole('article', { name: 'Data Engineer' })).not.toBeInTheDocument());
  });

  it('deletes a job only after confirming', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    const item = await screen.findByRole('article', { name: 'Data Engineer' });
    await userEvent.click(within(item).getByRole('button', { name: 'Excluir' }));
    expect(api.remove).not.toHaveBeenCalled();
    const dialog = await screen.findByRole('dialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancelar' }));
    expect(api.remove).not.toHaveBeenCalled();
    await userEvent.click(within(item).getByRole('button', { name: 'Excluir' }));
    await userEvent.click(
      within(await screen.findByRole('dialog')).getByRole('button', { name: 'Excluir definitivamente' }),
    );
    expect(api.remove).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
  });

  it('opens the job link by id', async () => {
    const api = fakeDiscarded([job()]);
    render(<DiscardedPage api={api} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Abrir vaga' }));
    expect(api.openLink).toHaveBeenCalledWith('aaaaaaaaaaaaaaaa');
  });

  it('shows the empty state', async () => {
    render(<DiscardedPage api={fakeDiscarded([])} />);
    expect(await screen.findByText('Nenhuma vaga descartada.')).toBeInTheDocument();
  });

  it('reports a failed action', async () => {
    const api = fakeDiscarded([job()]);
    vi.mocked(api.restore).mockRejectedValueOnce('storage error');
    render(<DiscardedPage api={api} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Mover para a Caixa de entrada' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível concluir a ação.');
    expect(screen.getByRole('article', { name: 'Data Engineer' })).toBeInTheDocument();
  });

  it('has no axe violations', async () => {
    const { container } = render(<DiscardedPage api={fakeDiscarded([job(), other])} />);
    await screen.findByRole('heading', { name: 'Data Engineer' });
    await expectNoAxeViolations(container);
  });

  it('reports a link that cannot open', async () => {
    const api = fakeDiscarded([job()]);
    vi.mocked(api.openLink).mockRejectedValueOnce('invalid_url');
    render(<DiscardedPage api={api} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Abrir vaga' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível abrir o link.');
  });
});
