import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { expectNoAxeViolations } from '@/test/axe';
import { CrawlerCard } from './CrawlerCard';

const base = { running: false, progress: null, error: null, run: null, onRun: vi.fn() };
const finished = {
  startedAt: '2026-11-02T10:00:00-03:00',
  finishedAt: '2026-11-02T10:00:40-03:00',
  stats: JSON.stringify({
    found: 50,
    inbox: 3,
    discarded: 4,
    sources: { gupy: { found: 30, error: null }, solides: { found: 20, error: 'x' } },
  }),
};

describe('CrawlerCard', () => {
  it('disables the button while running', () => {
    render(<CrawlerCard {...base} running progress="gupy: 3/10" />);
    expect(screen.getByRole('button', { name: 'Executando...' })).toBeDisabled();
    expect(screen.getByRole('status')).toHaveTextContent('gupy: 3/10');
  });

  it('runs on click', async () => {
    const onRun = vi.fn();
    render(<CrawlerCard {...base} onRun={onRun} />);
    await userEvent.click(screen.getByRole('button', { name: 'Executar agora' }));
    expect(onRun).toHaveBeenCalledOnce();
  });

  it('shows the real last run', () => {
    render(<CrawlerCard {...base} run={finished} />);
    expect(screen.getByText(/50 vagas/)).toBeInTheDocument();
    expect(screen.getByText(/40s/)).toBeInTheDocument();
    expect(screen.getByText(/1\/2 fontes ok/)).toBeInTheDocument();
    expect(screen.queryByText(/Próxima/)).not.toBeInTheDocument();
  });

  it('names the source that failed and why', () => {
    render(<CrawlerCard {...base} run={finished} />);
    expect(screen.getByText('solides: x')).toBeInTheDocument();
    expect(screen.queryByText(/^gupy:/)).not.toBeInTheDocument();
  });

  it('explains a never run and a refused run', () => {
    const { rerender } = render(<CrawlerCard {...base} />);
    expect(screen.getByText('O crawler ainda não rodou.')).toBeInTheDocument();
    rerender(<CrawlerCard {...base} error="config_missing" />);
    expect(screen.getByRole('alert')).toHaveTextContent('crawler.json');
    rerender(<CrawlerCard {...base} error="config_invalid:min_score" />);
    expect(screen.getByRole('alert')).toHaveTextContent('min_score');
  });

  it('never shows a raw error from the backend', () => {
    render(<CrawlerCard {...base} error="SQL detail at /some/path" />);
    expect(screen.getByRole('alert')).toHaveTextContent('Não foi possível iniciar o crawler.');
    expect(screen.queryByText(/SQL detail/)).not.toBeInTheDocument();
  });

  it('marks an interrupted run', () => {
    render(<CrawlerCard {...base} run={{ ...finished, finishedAt: null }} />);
    expect(screen.getByText('A última execução foi interrompida.')).toBeInTheDocument();
  });

  it('does not call the run in progress interrupted', () => {
    render(<CrawlerCard {...base} running run={{ ...finished, finishedAt: null }} />);
    expect(screen.queryByText('A última execução foi interrompida.')).not.toBeInTheDocument();
  });

  it('survives stats it cannot read', () => {
    render(<CrawlerCard {...base} run={{ ...finished, stats: '{ nope' }} />);
    expect(screen.getByRole('button', { name: 'Executar agora' })).toBeEnabled();
  });

  it('has no accessibility violations', async () => {
    const { container } = render(<CrawlerCard {...base} run={finished} error="config_missing" />);
    await expectNoAxeViolations(container);
  });
});
