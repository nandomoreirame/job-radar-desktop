import { Sparkles } from 'lucide-react';
import { useState, type FormEvent } from 'react';
import { useTranslation } from 'react-i18next';
import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { CardDetails } from '@/shared/bindings/CardDetails';
import { Button } from '@/shared/ui/button';
import { Field, FieldError, FieldLabel, FieldLegend, FieldSet } from '@/shared/ui/field';
import { Input } from '@/shared/ui/input';
import {
  ResponsiveDialog,
  ResponsiveDialogClose,
  ResponsiveDialogContent,
  ResponsiveDialogDescription,
  ResponsiveDialogFooter,
  ResponsiveDialogHeader,
  ResponsiveDialogTitle,
} from '@/shared/ui/responsive-dialog';
import { Textarea } from '@/shared/ui/textarea';
import {
  parseFieldError,
  toDetails,
  toForm,
  type DetailField,
  type DetailsForm,
  type FieldError as DetailFieldError,
} from '../model/details-form';

type SaveDetails = (details: CardDetails) => Promise<string | null>;
type Props = { card: BoardCard | null; onOpenChange: (open: boolean) => void; onSave: SaveDetails };

const ARTIFACTS = ['cv', 'coverage', 'interviewPrep'] as const;
const INPUT_TYPE: Partial<Record<DetailField, string>> = {
  nextActionDate: 'date',
  salaryPj: 'number',
  salaryClt: 'number',
  recruiterEmail: 'email',
  recruiterLinkedin: 'url',
  jobUrl: 'url',
  applicationUrl: 'url',
};
const SECTIONS: { key: string; fields: DetailField[] }[] = [
  { key: 'process', fields: ['processStep', 'nextAction', 'nextActionDate'] },
  { key: 'salary', fields: ['salaryPj', 'salaryClt'] },
  { key: 'recruiter', fields: ['recruiterName', 'recruiterEmail', 'recruiterLinkedin'] },
  { key: 'links', fields: ['jobUrl', 'applicationUrl'] },
];

export function JobDialog({ card, onOpenChange, onSave }: Props) {
  return (
    <ResponsiveDialog open={card !== null} onOpenChange={open => onOpenChange(open)}>
      {card && (
        <ResponsiveDialogContent
          showCloseButton={false}
          overlayClassName="backdrop-blur-none supports-backdrop-filter:backdrop-blur-none"
          className="max-h-[85vh] overflow-y-auto bg-noir-card sm:max-w-[56rem]"
        >
          {/* Keyed so the form resets when another card opens. */}
          <JobForm key={card.id} card={card} onSave={onSave} onDone={() => onOpenChange(false)} />
        </ResponsiveDialogContent>
      )}
    </ResponsiveDialog>
  );
}

function JobForm({ card, onSave, onDone }: { card: BoardCard; onSave: SaveDetails; onDone: () => void }) {
  const { t } = useTranslation();
  const [form, setForm] = useState<DetailsForm>(() => toForm(card));
  const [fieldError, setFieldError] = useState<DetailFieldError | null>(null);
  const [saveError, setSaveError] = useState(false);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState(false);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setSaving(true);
    const error = await onSave(toDetails(form));
    setSaving(false);
    if (error === null) return onDone();
    const parsed = parseFieldError(error);
    setFieldError(parsed);
    setSaveError(parsed === null);
  };

  const renderField = (field: DetailField) => {
    const id = `job-${field}`;
    const invalid = fieldError?.field === field;
    return (
      <Field key={field} data-invalid={invalid || undefined}>
        <FieldLabel htmlFor={id}>{t(`dialog.fields.${field}`)}</FieldLabel>
        <Input
          id={id}
          type={INPUT_TYPE[field] ?? 'text'}
          min={INPUT_TYPE[field] === 'number' ? 0 : undefined}
          value={form[field]}
          aria-invalid={invalid || undefined}
          aria-describedby={invalid ? `${id}-error` : undefined}
          onChange={event => setForm(current => ({ ...current, [field]: event.target.value }))}
          className={INPUT_TYPE[field] === 'number' ? 'font-mono' : undefined}
        />
        {invalid && <FieldError id={`${id}-error`}>{t(`dialog.errors.${fieldError.reason}`)}</FieldError>}
      </Field>
    );
  };

  return (
    <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-5">
      <ResponsiveDialogHeader>
        <ResponsiveDialogTitle>{card.title}</ResponsiveDialogTitle>
        <ResponsiveDialogDescription>
          {[card.company, card.source, t(`target.${card.target}`), t(`workModel.${card.workModel}`)].join(' · ')}
        </ResponsiveDialogDescription>
      </ResponsiveDialogHeader>

      <div className="grid gap-5 md:grid-cols-2">
        {SECTIONS.map(({ key, fields }) => (
          <FieldSet key={key}>
            <FieldLegend variant="label" className="text-xs uppercase text-noir-muted">
              {t(`dialog.sections.${key}`)}
            </FieldLegend>
            <div className="grid gap-3 sm:grid-cols-2">{fields.map(renderField)}</div>
          </FieldSet>
        ))}
      </div>

      <Field data-invalid={fieldError?.field === 'notes' || undefined}>
        <FieldLabel htmlFor="job-notes">{t('dialog.fields.notes')}</FieldLabel>
        <Textarea
          id="job-notes"
          rows={5}
          value={form.notes}
          aria-invalid={fieldError?.field === 'notes' || undefined}
          aria-describedby={fieldError?.field === 'notes' ? 'job-notes-error' : undefined}
          onChange={event => setForm(current => ({ ...current, notes: event.target.value }))}
        />
        {fieldError?.field === 'notes' && (
          <FieldError id="job-notes-error">{t(`dialog.errors.${fieldError.reason}`)}</FieldError>
        )}
      </Field>

      <div className="grid gap-5 text-sm md:grid-cols-2">
        <section aria-labelledby="job-dialog-artifacts" className="flex flex-col gap-1">
          <h3 id="job-dialog-artifacts" className="text-xs font-semibold uppercase text-noir-muted">
            {t('dialog.artifacts')}
          </h3>
          <ul>
            {ARTIFACTS.map(key => (
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
              <li key={`${event.at}-${event.kind}`} className="flex gap-3">
                <time dateTime={event.at} className="font-mono text-noir-muted">
                  {event.at}
                </time>
                <span>{event.label}</span>
              </li>
            ))}
          </ol>
        </section>
      </div>

      {saveError && (
        <p role="alert" className="text-sm text-noir-red">
          {t('dialog.saveError')}
        </p>
      )}

      <ResponsiveDialogFooter className="items-center gap-3">
        {notice && (
          <p role="status" className="mr-auto text-xs text-noir-muted">
            {t('app.notImplemented')}
          </p>
        )}
        {/* CV tailoring starts from a fresh job; the AI CLI runner comes in a later plan. */}
        {card.stage === 'inbox' && (
          <Button type="button" variant="outline" onClick={() => setNotice(true)}>
            <Sparkles aria-hidden="true" />
            {t('dialog.generateCv')}
          </Button>
        )}
        {/* ResponsiveDialogClose drops children when given `render`, so style it directly. */}
        <ResponsiveDialogClose className="inline-flex h-9 items-center justify-center rounded-md px-3 text-sm font-medium hover:bg-white/5">
          {t('dialog.cancel')}
        </ResponsiveDialogClose>
        <Button type="submit" disabled={saving}>
          {t('dialog.save')}
        </Button>
      </ResponsiveDialogFooter>
    </form>
  );
}
