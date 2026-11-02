import { ExternalLink, RotateCcw, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDiscarded } from '@/entities/job/model/use-discarded';
import { matchesText } from '@/features/filter-board/model/filter';
import { discardedApi, type DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/shared/ui/dialog';
import { Input } from '@/shared/ui/input';
type Props = { api?: DiscardedApi };
export function DiscardedPage({ api = discardedApi }: Props) {
  const { t } = useTranslation();
  const { jobs, status, actionError, restore, remove } = useDiscarded(api);
  const [q, setQ] = useState('');
  const [confirming, setConfirming] = useState<DiscardedJob | null>(null);
  const visible = useMemo(() => jobs.filter(j => matchesText(`${j.title} ${j.company} ${j.source}`, q)), [jobs, q]);
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
      <div className="flex flex-wrap items-center gap-4">
        <h1 className="h-9 text-2xl font-semibold leading-9">{t('discarded.title')}</h1>
        <Input
          type="search"
          aria-label={t('discarded.search')}
          placeholder={t('discarded.search')}
          value={q}
          onChange={e => setQ(e.target.value)}
          className="h-9 w-72"
        />
      </div>
      {actionError && (
        <p role="alert" className="text-sm text-noir-red">
          {t('discarded.actionError')}
        </p>
      )}
      {status === 'loading' && <p role="status">{t('board.loading')}</p>}
      {status === 'error' && <p role="alert">{t('board.error')}</p>}
      {status === 'ready' && jobs.length === 0 && <p>{t('discarded.empty')}</p>}
      <ul className="flex flex-col gap-3">
        {visible.map(job => (
          <li key={job.id}>
            <article aria-labelledby={`discarded-${job.id}`} className="noir-card flex flex-col gap-2 rounded-lg p-4">
              <h2 id={`discarded-${job.id}`} className="text-sm font-semibold">
                {job.title}
              </h2>
              <p className="text-sm text-noir-muted">
                {[job.company, job.source, job.location].filter(Boolean).join(' · ')}
              </p>
              <p className="text-xs text-noir-muted">
                {job.reason} · <time dateTime={job.discardedAt}>{job.discardedAt}</time>
              </p>
              {job.description && (
                <details>
                  <summary className="cursor-pointer text-xs">{t('discarded.description')}</summary>
                  <p className="mt-2 max-h-64 overflow-y-auto whitespace-pre-wrap text-sm">{job.description}</p>
                </details>
              )}
              <div className="flex flex-wrap gap-2">
                <Button type="button" variant="outline" size="sm" onClick={() => void restore(job.id)}>
                  <RotateCcw aria-hidden="true" />
                  {t('discarded.restore')}
                </Button>
                {job.jobUrl && (
                  <Button type="button" variant="outline" size="sm" onClick={() => void api.openLink(job.id)}>
                    <ExternalLink aria-hidden="true" />
                    {t('dialog.openLink')}
                  </Button>
                )}
                <Button type="button" variant="outline" size="sm" onClick={() => setConfirming(job)}>
                  <Trash2 aria-hidden="true" />
                  {t('discarded.delete')}
                </Button>
              </div>
            </article>
          </li>
        ))}
      </ul>
      <Dialog open={confirming !== null} onOpenChange={open => !open && setConfirming(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('discarded.confirmTitle')}</DialogTitle>
            <DialogDescription>{t('discarded.confirmBody', { title: confirming?.title })}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setConfirming(null)}>
              {t('discarded.cancel')}
            </Button>
            <Button
              type="button"
              variant="destructive"
              onClick={() => {
                if (confirming) void remove(confirming.id);
                setConfirming(null);
              }}
            >
              {t('discarded.confirmDelete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
