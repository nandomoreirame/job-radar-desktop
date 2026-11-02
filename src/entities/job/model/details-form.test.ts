import { describe, expect, it } from 'vitest';
import { EMPTY_DETAILS, makeCard } from '@/test/factories';
import { parseFieldError, toDetails, toForm } from './details-form';

describe('toForm', () => {
  it('fills every field from the card', () => {
    const form = toForm(
      makeCard({
        salary: { pj: 28_000, clt: null },
        processStep: 'Técnica',
        recruiterEmail: 'ana@example.com',
        notes: 'Ligar na sexta',
      }),
    );
    expect(form.salaryPj).toBe('28000');
    expect(form.salaryClt).toBe('');
    expect(form.processStep).toBe('Técnica');
    expect(form.recruiterEmail).toBe('ana@example.com');
    expect(form.notes).toBe('Ligar na sexta');
  });
});

describe('toDetails', () => {
  it('maps empty fields to null and keeps notes as text', () => {
    expect(toDetails(toForm(makeCard()))).toEqual(EMPTY_DETAILS);
  });

  it('parses the salary in whole reais', () => {
    const form = { ...toForm(makeCard()), salaryPj: '30000', salaryClt: '22500.7' };
    expect(toDetails(form).salary).toEqual({ pj: 30_000, clt: 22_501 });
  });

  it('drops a salary that is not a positive number', () => {
    const form = { ...toForm(makeCard()), salaryPj: '-5', salaryClt: 'abc' };
    expect(toDetails(form).salary).toBeNull();
  });
});

describe('parseFieldError', () => {
  it('reads the field and reason from the store error', () => {
    expect(parseFieldError('jobUrl: invalid_url')).toEqual({ field: 'jobUrl', reason: 'invalid_url' });
    expect(parseFieldError('salaryPj: out_of_range')).toEqual({ field: 'salaryPj', reason: 'out_of_range' });
  });

  it('ignores errors that are not about a form field', () => {
    expect(parseFieldError('card not found: c99')).toBeNull();
    expect(parseFieldError('title: too_long')).toBeNull();
  });
});
