import { Play } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { CrawlRun } from '@/shared/bindings/CrawlRun';

type SourceStats = { found?: number; error?: string | null };
type RunStats = { found?: number; inbox?: number; sources?: Record<string, SourceStats> };

function parseStats(raw: string): RunStats | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    return parsed !== null && typeof parsed === 'object' ? (parsed as RunStats) : null;
  } catch {
    return null;
  }
}

function secondsBetween(start: string, end: string | null): number | null {
  if (end === null) return null;
  const seconds = Math.round((Date.parse(end) - Date.parse(start)) / 1000);
  return Number.isFinite(seconds) ? seconds : null;
}

/** Rust refuses with a stable code, `code` or `code:field`; anything else stays generic. */
function errorParts(error: string): { code: string; field: string } {
  const [code, field = ''] = error.split(':', 2);
  return { code, field };
}

type Props = {
  running: boolean;
  progress: string | null;
  error: string | null;
  run: CrawlRun | null;
  onRun: () => void;
};

export function CrawlerCard({ running, progress, error, run, onRun }: Props) {
  const { t, i18n } = useTranslation();
  const label = running ? t('crawler.running') : t('crawler.runNow');
  const finished = run !== null && run.finishedAt !== null ? run : null;
  const stats = finished ? parseStats(finished.stats) : null;
  const sources = Object.entries(stats?.sources ?? {});
  const failed = sources.filter(([, source]) => source.error !== null);
  const duration = finished ? secondsBetween(finished.startedAt, finished.finishedAt) : null;
  const refused = error === null ? null : errorParts(error);
  const time = finished
    ? new Date(finished.startedAt).toLocaleTimeString(i18n.language, { hour: '2-digit', minute: '2-digit' })
    : '';
  return (
    <section
      aria-label={t('crawler.title')}
      className="flex flex-col gap-1 rounded-lg border border-noir-border bg-noir-card/70 p-3 text-xs text-noir-muted group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:items-center group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
    >
      <p className={`flex items-center gap-2 ${running ? 'text-noir-amber' : 'text-noir-teal'}`}>
        <span
          aria-hidden="true"
          className={`size-2 shrink-0 rounded-full ${running ? 'animate-pulse bg-noir-amber' : 'bg-noir-teal'}`}
        />
        <span className="group-data-[collapsible=icon]:sr-only">{t('crawler.title')}</span>
      </p>
      <div className="flex flex-col gap-1 group-data-[collapsible=icon]:hidden">
        {run === null && !running && <p>{t('crawler.never')}</p>}
        {run !== null && run.finishedAt === null && !running && <p>{t('crawler.interrupted')}</p>}
        {stats && <p>{t('crawler.lastRun', { time, total: stats.found ?? 0, fresh: stats.inbox ?? 0 })}</p>}
        {duration !== null && <p>{t('crawler.duration', { duration })}</p>}
        {sources.length > 0 && (
          <p>{t('crawler.sources', { ok: sources.length - failed.length, total: sources.length })}</p>
        )}
        {failed.map(([name, source]) => (
          <p key={name} className="break-words text-noir-amber">
            {`${name}: ${source.error ?? ''}`}
          </p>
        ))}
      </div>
      <button
        type="button"
        title={label}
        disabled={running}
        onClick={onRun}
        className="mt-2 flex items-center gap-2 rounded-md border border-noir-border px-2 py-1.5 text-left text-noir-text hover:border-noir-amber hover:text-noir-amber focus-visible:outline-2 focus-visible:outline-noir-amber disabled:cursor-not-allowed disabled:opacity-60 group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:p-1"
      >
        <Play aria-hidden="true" className="size-3 shrink-0" />
        <span className="group-data-[collapsible=icon]:sr-only">{label}</span>
      </button>
      {running && progress !== null && (
        <p role="status" className="break-words group-data-[collapsible=icon]:sr-only">
          {progress}
        </p>
      )}
      {refused && (
        <p role="alert" className="break-words text-noir-red group-data-[collapsible=icon]:sr-only">
          {t(`crawler.errors.${refused.code}`, {
            field: refused.field,
            defaultValue: t('crawler.errors.unknown'),
          })}
        </p>
      )}
    </section>
  );
}
