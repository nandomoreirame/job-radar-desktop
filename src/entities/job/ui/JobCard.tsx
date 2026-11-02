import { FileText, ListChecks, MessagesSquare } from 'lucide-react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import { cn } from '@/shared/lib/utils';
import { Badge } from '@/shared/ui/badge';
import { Status, StatusIndicator, StatusLabel } from '@/shared/ui/status';
import { outcomeVariant, scoreTone } from '../model/format';
import { SalaryLine } from './SalaryLine';

const SCORE_TONE = {
  high: 'border-noir-teal/60 text-noir-teal',
  good: 'border-noir-amber/60 text-noir-amber',
  low: 'border-noir-border text-noir-muted',
} as const;

const capitalize = (value: string) => value.charAt(0).toUpperCase() + value.slice(1);

type Props = {
  card: BoardCard;
  onOpen: (card: BoardCard) => void;
  /** Slot for card actions (the move menu); sits above the stretched title button. */
  actions?: ReactNode;
};

export function JobCard({ card, onOpen, actions }: Props) {
  const { t } = useTranslation();
  const titleId = `job-${card.id}-title`;
  const artifacts = [
    { key: 'cv', done: card.artifacts.cv, Icon: FileText },
    { key: 'coverage', done: card.artifacts.coverage, Icon: ListChecks },
    { key: 'interviewPrep', done: card.artifacts.interviewPrep, Icon: MessagesSquare },
  ] as const;
  const note = card.nextAction ?? card.lastActivity;

  return (
    <article aria-labelledby={titleId} className="noir-card relative flex flex-col gap-3 rounded-lg p-3">
      <div className="flex items-start gap-3">
        <span
          aria-hidden="true"
          className="grid size-7 shrink-0 place-items-center rounded-md bg-white/5 font-mono text-xs text-noir-muted"
        >
          {card.company.charAt(0)}
        </span>
        <div className="min-w-0 flex-1">
          <h3 id={titleId} className="text-sm font-semibold leading-snug">
            {/* Stretched over the whole card so a click anywhere opens the dialog. */}
            <button
              type="button"
              onClick={() => onOpen(card)}
              aria-label={t('card.open', { title: card.title })}
              className="text-left after:absolute after:inset-0 after:rounded-lg focus-visible:outline-none"
            >
              {card.title}
            </button>
          </h3>
          <p className="truncate text-sm text-noir-muted">{card.company}</p>
        </div>
        {card.unread && (
          <span
            role="img"
            aria-label={t('card.unread')}
            className="mt-1.5 size-2 shrink-0 rounded-full bg-noir-amber"
          />
        )}
        <span className={cn('rounded-md border px-1.5 py-0.5 font-mono text-xs', SCORE_TONE[scoreTone(card.score)])}>
          <span className="sr-only">{`${t('card.scoreLabel')} ${card.score}`}</span>
          <span aria-hidden="true">{card.score}</span>
        </span>
      </div>

      {card.outcome && (
        <Status variant={outcomeVariant(card.outcome)}>
          <StatusLabel>{t(`outcome.${card.outcome}`)}</StatusLabel>
        </Status>
      )}

      <div className="flex flex-wrap items-center gap-1.5">
        <Badge variant="secondary">{card.source}</Badge>
        <Badge variant="secondary" className="text-noir-amber">
          {t(`target.${card.target}`)}
        </Badge>
        <span className="text-xs text-noir-muted">{t(`workModel.${card.workModel}`)}</span>
        {card.location && <span className="text-xs text-noir-muted">· {card.location}</span>}
        {card.processStep && (
          <Status variant="warning" className="font-mono uppercase">
            <StatusLabel>{card.processStep}</StatusLabel>
          </Status>
        )}
      </div>

      <SalaryLine salary={card.salary} />

      {card.aiRunning ? (
        <Status role="status" variant="info" className="w-full">
          <StatusIndicator />
          <StatusLabel>{t('card.aiRunning', { cli: capitalize(card.aiRunning) })}</StatusLabel>
        </Status>
      ) : (
        <ul aria-label={t('card.artifacts.label')} className="flex gap-2">
          {artifacts.map(({ key, done, Icon }) => (
            <li key={key}>
              <Icon
                role="img"
                aria-label={`${t(`card.artifacts.${key}`)}: ${t(done ? 'card.artifacts.done' : 'card.artifacts.missing')}`}
                className={cn('size-4', done ? 'text-noir-teal' : 'text-noir-muted/40')}
              />
            </li>
          ))}
        </ul>
      )}

      {note && <p className="text-xs text-noir-muted">{note}</p>}
      {actions && <div className="relative z-10 flex items-center gap-1 self-end">{actions}</div>}
    </article>
  );
}
