import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useBoard } from '@/entities/job/model/use-board';
import { JobDialog } from '@/entities/job/ui/JobDialog';
import { DEFAULT_FILTERS, filterCards, type BoardFilters } from '@/features/filter-board/model/filter';
import { Toolbar } from '@/features/filter-board/ui/Toolbar';
import { JobCommand } from '@/features/search-jobs/ui/JobCommand';
import { boardApi, type BoardApi } from '@/shared/api/board';
import { useHotkey } from '@/shared/lib/hooks/use-hotkey';
import { Board } from '@/widgets/board/Board';
import { MetricsStrip } from '@/widgets/board/MetricsStrip';
import { computeMetrics } from '@/widgets/board/model/metrics';

type Props = { api?: BoardApi; now?: Date };

export function BoardPage({ api = boardApi, now }: Props) {
  const { t } = useTranslation();
  const { cards, status, moveError, move, update } = useBoard(api);
  const [filters, setFilters] = useState<BoardFilters>(DEFAULT_FILTERS);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const visible = useMemo(() => filterCards(cards, filters), [cards, filters]);
  // Metrics follow the filters, so a source or target can be judged on its own.
  const metrics = useMemo(() => computeMetrics(visible, now ?? new Date()), [visible, now]);
  const sources = useMemo(() => [...new Set(cards.map(card => card.source))].sort(), [cards]);
  const [openId, setOpenId] = useState<string | null>(null);
  const openCard = cards.find(card => card.id === openId) ?? null;
  const openPalette = useCallback(() => setPaletteOpen(true), []);
  useHotkey('k', openPalette);

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden pt-6">
      <div className="flex flex-wrap items-center gap-4 px-6">
        <h1 className="h-9 text-2xl font-semibold leading-9">{t('board.title')}</h1>
        <Toolbar filters={filters} sources={sources} onChange={setFilters} onSearch={openPalette} />
      </div>
      <div className="px-6">
        <MetricsStrip metrics={metrics} />
      </div>
      {moveError && (
        <p role="alert" className="px-6 text-sm text-noir-red">
          {t('board.moveError')}
        </p>
      )}
      {status === 'loading' && (
        <p role="status" className="px-6">
          {t('board.loading')}
        </p>
      )}
      {status === 'error' && (
        <p role="alert" className="px-6">
          {t('board.error')}
        </p>
      )}
      {status === 'ready' && (
        <Board
          cards={visible}
          onOpen={card => setOpenId(card.id)}
          onMove={(id, stage, outcome) => void move(id, stage, outcome)}
        />
      )}
      <JobCommand cards={cards} open={paletteOpen} onOpenChange={setPaletteOpen} onPick={card => setOpenId(card.id)} />
      <JobDialog
        card={openCard}
        onOpenChange={open => !open && setOpenId(null)}
        onSave={details => (openCard ? update(openCard.id, details) : Promise.resolve(null))}
        onOpenLink={id => void api.openLink(id)}
      />
    </div>
  );
}
