import { describe, expect, it } from 'vitest';
import { formatThousands, hasNegotiatedSalary, outcomeVariant, scoreTone } from './format';

describe('formatThousands', () => {
  it('rounds monthly pay to thousands', () => {
    expect(formatThousands(28_000)).toBe('28');
    expect(formatThousands(21_500)).toBe('22');
  });
});

describe('scoreTone', () => {
  it('splits scores into three tones', () => {
    expect(scoreTone(96)).toBe('high');
    expect(scoreTone(85)).toBe('good');
    expect(scoreTone(70)).toBe('low');
  });
});

describe('hasNegotiatedSalary', () => {
  it('needs at least one regime', () => {
    expect(hasNegotiatedSalary(null)).toBe(false);
    expect(hasNegotiatedSalary({ pj: null, clt: null })).toBe(false);
    expect(hasNegotiatedSalary({ pj: null, clt: 18_000 })).toBe(true);
  });
});

describe('outcomeVariant', () => {
  it('maps each outcome to a status variant', () => {
    expect(outcomeVariant('hired')).toBe('success');
    expect(outcomeVariant('rejected')).toBe('error');
    expect(outcomeVariant('discarded')).toBe('default');
    expect(outcomeVariant('no_response')).toBe('default');
  });
});
