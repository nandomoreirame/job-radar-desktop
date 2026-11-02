import { Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import { Button } from '@/shared/ui/button';
import {
  ResponsiveDialog,
  ResponsiveDialogClose,
  ResponsiveDialogContent,
  ResponsiveDialogDescription,
  ResponsiveDialogFooter,
  ResponsiveDialogHeader,
  ResponsiveDialogTitle,
} from '@/shared/ui/responsive-dialog';
import { hasNegotiatedSalary } from '../model/format';
import { SalaryLine } from './SalaryLine';

type Props = { card: BoardCard | null; onOpenChange: (open: boolean) => void };

export function JobDialog({ card, onOpenChange }: Props) {
  const { t } = useTranslation();
  return (
    <ResponsiveDialog open={card !== null} onOpenChange={open => onOpenChange(open)}>
      {card && (
        <ResponsiveDialogContent showCloseButton={false} className="bg-noir-card sm:max-w-[42rem]">
          <ResponsiveDialogHeader>
            <ResponsiveDialogTitle>{card.title}</ResponsiveDialogTitle>
            <ResponsiveDialogDescription>
              {[card.company, card.source, t(`target.${card.target}`), t(`workModel.${card.workModel}`)].join(' · ')}
            </ResponsiveDialogDescription>
          </ResponsiveDialogHeader>

          <section aria-labelledby="job-dialog-salary" className="flex flex-col gap-1">
            <h3 id="job-dialog-salary" className="text-xs font-semibold uppercase text-noir-muted">
              {t('dialog.salary')}
            </h3>
            {hasNegotiatedSalary(card.salary) ? (
              <SalaryLine salary={card.salary} />
            ) : (
              <p className="text-sm text-noir-muted">{t('dialog.noSalary')}</p>
            )}
          </section>

          <section aria-labelledby="job-dialog-artifacts" className="flex flex-col gap-1">
            <h3 id="job-dialog-artifacts" className="text-xs font-semibold uppercase text-noir-muted">
              {t('dialog.artifacts')}
            </h3>
            <ul className="text-sm">
              {(['cv', 'coverage', 'interviewPrep'] as const).map(key => (
                <li key={key}>
                  {t(`card.artifacts.${key}`)}:{' '}
                  {t(card.artifacts[key] ? 'card.artifacts.done' : 'card.artifacts.missing')}
                </li>
              ))}
            </ul>
          </section>

          <section aria-labelledby="job-dialog-timeline" className="flex flex-col gap-1">
            <h3 id="job-dialog-timeline" className="text-xs font-semibold uppercase text-noir-muted">
              {t('dialog.timeline')}
            </h3>
            <ol className="flex flex-col gap-1">
              {card.timeline.map(event => (
                <li key={`${event.at}-${event.kind}`} className="flex gap-3 text-sm">
                  <time dateTime={event.at} className="font-mono text-noir-muted">
                    {event.at}
                  </time>
                  <span>{event.label}</span>
                </li>
              ))}
            </ol>
          </section>

          <ResponsiveDialogFooter className="items-center gap-3">
            <span id="job-dialog-cv-hint" className="text-xs text-noir-muted">
              {t('dialog.comingSoon')}
            </span>
            {/* Enabled once the AI CLI runner exists (later plan). */}
            <Button disabled aria-describedby="job-dialog-cv-hint">
              <Sparkles aria-hidden="true" />
              {t('dialog.generateCv')}
            </Button>
            <ResponsiveDialogClose className="inline-flex h-9 items-center justify-center rounded-md px-3 text-sm font-medium hover:bg-white/5">
              {t('dialog.close')}
            </ResponsiveDialogClose>
          </ResponsiveDialogFooter>
        </ResponsiveDialogContent>
      )}
    </ResponsiveDialog>
  );
}
