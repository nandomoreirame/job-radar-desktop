import type { BoardCard } from '@/shared/bindings/BoardCard';
import type { CardDetails } from '@/shared/bindings/CardDetails';

/** Form fields the user edits; the Rust side names them the same in its errors. */
export const DETAIL_FIELDS = [
  'processStep',
  'nextAction',
  'nextActionDate',
  'salaryPj',
  'salaryClt',
  'recruiterName',
  'recruiterEmail',
  'recruiterLinkedin',
  'jobUrl',
  'applicationUrl',
  'notes',
] as const;

export type DetailField = (typeof DETAIL_FIELDS)[number];
export type DetailsForm = Record<DetailField, string>;
export type FieldError = { field: DetailField; reason: string };

export function toForm(card: BoardCard): DetailsForm {
  return {
    processStep: card.processStep ?? '',
    nextAction: card.nextAction ?? '',
    nextActionDate: card.nextActionDate ?? '',
    salaryPj: card.salary?.pj?.toString() ?? '',
    salaryClt: card.salary?.clt?.toString() ?? '',
    recruiterName: card.recruiterName ?? '',
    recruiterEmail: card.recruiterEmail ?? '',
    recruiterLinkedin: card.recruiterLinkedin ?? '',
    jobUrl: card.jobUrl ?? '',
    applicationUrl: card.applicationUrl ?? '',
    notes: card.notes,
  };
}

const orNull = (value: string) => (value.trim() === '' ? null : value);

/** Whole reais; empty, negative or non-numeric input means "not negotiated". */
function money(raw: string): number | null {
  const value = Number(raw);
  if (raw.trim() === '' || !Number.isFinite(value) || value <= 0) return null;
  return Math.round(value);
}

export function toDetails(form: DetailsForm): CardDetails {
  const pj = money(form.salaryPj);
  const clt = money(form.salaryClt);
  return {
    salary: pj === null && clt === null ? null : { pj, clt },
    processStep: orNull(form.processStep),
    nextAction: orNull(form.nextAction),
    nextActionDate: orNull(form.nextActionDate),
    recruiterName: orNull(form.recruiterName),
    recruiterEmail: orNull(form.recruiterEmail),
    recruiterLinkedin: orNull(form.recruiterLinkedin),
    jobUrl: orNull(form.jobUrl),
    applicationUrl: orNull(form.applicationUrl),
    // Validation (trim, length) is the Rust side's job; send what was typed.
    notes: form.notes,
  };
}

/** Reads `update_card`'s `<field>: <reason>` error; null when it is not about a field. */
export function parseFieldError(message: string): FieldError | null {
  const match = /^(\w+): (\w+)$/.exec(message);
  if (!match) return null;
  const field = match[1] as DetailField;
  return DETAIL_FIELDS.includes(field) ? { field, reason: match[2] } : null;
}
