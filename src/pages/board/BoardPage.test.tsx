import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import type { BoardApi } from '@/shared/api/board';
import { fakeApi, makeCard } from '@/test/factories';
import { BoardPage } from './BoardPage';

const cards = [
  makeCard({ id: 'v', title: 'Lead AI Architect', company: 'Vértice Saúde' }),
  makeCard({ id: 'o', title: 'Tech Lead', company: 'Orbita Pay' }),
];
const now = new Date('2026-11-02T12:00:00Z');

describe('BoardPage', () => {
  it('filters the board to unread cards', async () => {
    const list = [makeCard({ id: 'u', title: 'Unread Job', unread: true }), makeCard({ id: 'r', title: 'Read Job' })];
    render(<BoardPage api={fakeApi(list)} now={now} />);
    await screen.findByRole('heading', { name: /Read Job/ });
    await userEvent.click(screen.getByRole('checkbox', { name: 'Somente não lidas' }));
    expect(screen.getByRole('heading', { name: /Unread Job/ })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: /^Read Job/ })).not.toBeInTheDocument();
  });

  it('shows the loading state until the api answers', () => {
    const api: BoardApi = { ...fakeApi([]), list: () => new Promise(() => {}) };
    render(<BoardPage api={api} now={now} />);
    expect(screen.getByRole('status')).toHaveTextContent('Carregando vagas…');
  });

  it('reports a failed load', async () => {
    const api: BoardApi = { ...fakeApi([]), list: async () => Promise.reject(new Error('boom')) };
    render(<BoardPage api={api} now={now} />);
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível carregar as vagas.');
  });

  it('opens the details of a card', async () => {
    render(<BoardPage api={fakeApi(cards)} now={now} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Abrir detalhes de Lead AI Architect' }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByRole('heading', { name: 'Lead AI Architect' })).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Fechar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  });

  it('moves a card to the chosen column', async () => {
    render(<BoardPage api={fakeApi(cards)} now={now} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Ações de Lead AI Architect' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Triagem' }));
    const triage = screen.getByRole('region', { name: 'Triagem' });
    expect(await within(triage).findByRole('heading', { name: /Lead AI Architect/ })).toBeInTheDocument();
  });

  it('reports a rejected move', async () => {
    const api: BoardApi = { ...fakeApi(cards), move: async () => Promise.reject(new Error('no')) };
    render(<BoardPage api={api} now={now} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Ações de Lead AI Architect' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Triagem' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível mover a vaga.');
  });
  it('opens a job from the command palette', async () => {
    render(<BoardPage api={fakeApi(cards)} now={now} />);
    await screen.findByRole('heading', { name: /Lead AI Architect/ });
    await userEvent.keyboard('{Control>}k{/Control}');
    await userEvent.type(await screen.findByRole('combobox'), 'vertice');
    await userEvent.click(screen.getByRole('option', { name: /Lead AI Architect/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Lead AI Architect' });
    expect(within(dialog).getByRole('button', { name: 'Salvar' })).toBeInTheDocument();
  });

  it('saves the notes of a card through the store', async () => {
    render(<BoardPage api={fakeApi(cards)} now={now} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Abrir detalhes de Lead AI Architect' }));
    const dialog = await screen.findByRole('dialog');
    await waitFor(() => expect(within(dialog).getByRole('button', { name: 'Fechar' })).toHaveFocus());
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Anotações' }), 'Ligar na sexta');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
    await userEvent.click(screen.getByRole('button', { name: 'Abrir detalhes de Lead AI Architect' }));
    expect(await screen.findByRole('textbox', { name: 'Anotações' })).toHaveValue('Ligar na sexta');
  });
});
it('shows the empty state with no cards', async () => {
  render(<BoardPage api={fakeApi([])} />);
  expect(await screen.findByText('Nenhuma vaga ainda')).toBeInTheDocument();
});
