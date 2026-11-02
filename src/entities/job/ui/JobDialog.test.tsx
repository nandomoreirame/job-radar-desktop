import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { makeCard } from '@/test/factories';
import { JobDialog } from './JobDialog';

const card = makeCard({
  stage: 'applied',
  salary: { pj: 28_000, clt: null },
  notes: 'Falar do ETL',
  timeline: [{ at: '2026-10-30', kind: 'applied', label: 'Candidatura enviada' }],
});
const saveOk = () => vi.fn(async () => null);

describe('JobDialog', () => {
  it('shows the job and prefills the form', () => {
    render(<JobDialog card={card} onOpenChange={() => {}} onSave={saveOk()} />);
    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByRole('heading', { name: 'Staff Engineer' })).toBeInTheDocument();
    expect(within(dialog).getByRole('textbox', { name: 'Anotações' })).toHaveValue('Falar do ETL');
    expect(within(dialog).getByRole('spinbutton', { name: 'PJ (R$)' })).toHaveValue(28000);
    expect(within(dialog).getByText('Candidatura enviada')).toBeInTheDocument();
  });

  it('saves the edited details and closes', async () => {
    const onSave = saveOk();
    const onOpenChange = vi.fn();
    render(<JobDialog card={card} onOpenChange={onOpenChange} onSave={onSave} />);
    // Base UI moves the initial focus into the dialog asynchronously; typing before
    // that lands would be split across fields.
    await waitFor(() => expect(screen.getByRole('button', { name: 'Fechar' })).toHaveFocus());
    const notes = screen.getByRole('textbox', { name: 'Anotações' });
    await userEvent.clear(notes);
    await userEvent.type(notes, 'Ligar na sexta');
    await userEvent.type(screen.getByRole('textbox', { name: 'Nome' }), 'Ana');
    await userEvent.click(screen.getByRole('button', { name: 'Salvar' }));
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ notes: 'Ligar na sexta', recruiterName: 'Ana', salary: { pj: 28000, clt: null } }),
    );
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('points at the field the store rejected', async () => {
    const onOpenChange = vi.fn();
    const onSave = vi.fn(async () => 'jobUrl: invalid_url');
    render(<JobDialog card={card} onOpenChange={onOpenChange} onSave={onSave} />);
    await userEvent.click(screen.getByRole('button', { name: 'Salvar' }));
    const field = screen.getByRole('textbox', { name: 'Link da vaga' });
    expect(field).toHaveAttribute('aria-invalid', 'true');
    expect(field).toHaveAccessibleDescription('Use um link http(s) válido.');
    expect(onOpenChange).not.toHaveBeenCalled();
  });

  it('reports a save error that is not about a field', async () => {
    const onSave = vi.fn(async () => 'card not found: c01');
    render(<JobDialog card={card} onOpenChange={() => {}} onSave={onSave} />);
    await userEvent.click(screen.getByRole('button', { name: 'Salvar' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Não foi possível salvar a vaga.');
  });

  it('offers CV generation only in the inbox', () => {
    const { unmount } = render(<JobDialog card={card} onOpenChange={() => {}} onSave={saveOk()} />);
    expect(screen.queryByRole('button', { name: 'Gerar CV customizado' })).not.toBeInTheDocument();
    unmount();
    render(<JobDialog card={makeCard({ stage: 'inbox' })} onOpenChange={() => {}} onSave={saveOk()} />);
    expect(screen.getByRole('button', { name: 'Gerar CV customizado' })).toBeEnabled();
  });

  it('tells that CV generation is not implemented yet', async () => {
    render(<JobDialog card={makeCard({ stage: 'inbox' })} onOpenChange={() => {}} onSave={saveOk()} />);
    await userEvent.click(screen.getByRole('button', { name: 'Gerar CV customizado' }));
    expect(screen.getByRole('status')).toHaveTextContent('Ainda não implementado.');
  });

  it('marks each artifact', () => {
    render(
      <JobDialog
        card={makeCard({ artifacts: { cv: true, coverage: false, interviewPrep: true } })}
        onOpenChange={() => {}}
        onSave={saveOk()}
      />,
    );
    expect(screen.getByText(/CV customizado: pronto/)).toBeInTheDocument();
    expect(screen.getByText(/Revisão de cobertura: pendente/)).toBeInTheDocument();
  });

  it('closes without saving from the close button at the top', async () => {
    const onSave = saveOk();
    const onOpenChange = vi.fn();
    render(<JobDialog card={card} onOpenChange={onOpenChange} onSave={onSave} />);
    expect(screen.queryByRole('button', { name: 'Cancelar' })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Fechar' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onSave).not.toHaveBeenCalled();
  });

  it('closes with Escape', async () => {
    const onOpenChange = vi.fn();
    render(<JobDialog card={card} onOpenChange={onOpenChange} onSave={saveOk()} />);
    await userEvent.keyboard('{Escape}');
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('renders the dialog variant on desktop', () => {
    render(<JobDialog card={card} onOpenChange={() => {}} onSave={saveOk()} />);
    expect(screen.getByRole('dialog')).toHaveAttribute('data-variant', 'dialog');
  });
});
