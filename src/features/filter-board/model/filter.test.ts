import { describe, expect, it } from 'vitest';
import { makeCard } from '@/test/factories';
import { DEFAULT_FILTERS, EMPTY_FILTERS, clampScore, filterCards, matchesText } from './filter';
const v = makeCard({
  id: 'v',
  title: 'Lead AI Architect',
  company: 'Vértice Saúde',
  source: 'Gmail',
  target: 'ai_llm',
  score: 91,
  unread: true,
});
const n = makeCard({
  id: 'n',
  title: 'Engenheiro de Dados',
  company: 'Nébula Data',
  source: 'Lever',
  target: 'data',
  score: 79,
});
const o = makeCard({
  id: 'o',
  title: 'Tech Lead',
  company: 'Orbita Pay',
  source: 'Gupy',
  target: 'tech_lead',
  score: 82,
});
const ids = (cards: { id: string }[]) => cards.map(c => c.id);
describe('filterCards', () => {
  it('keeps every card with empty filters', () =>
    expect(ids(filterCards([v, n, o], EMPTY_FILTERS))).toEqual(['v', 'n', 'o']));
  it('filters by source and target', () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ['Gupy'], targets: ['tech_lead'] }))).toEqual(['o']);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ['Lever', 'Gmail'] }))).toEqual(['v', 'n']);
  });
  it('drops cards below the minimum score', () =>
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, minScore: 80 }))).toEqual(['v', 'o']));
  it('keeps only unread cards when asked', () =>
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, unreadOnly: true }))).toEqual(['v']));
});
describe('matchesText', () => {
  it('matches titles and companies without accents', () => {
    expect(matchesText('Vértice Saúde', 'vertice')).toBe(true);
    expect(matchesText('Engenheiro de Dados Sênior', 'SENIOR')).toBe(true);
    expect(matchesText('Orbita Pay', 'vertice')).toBe(false);
  });
});
describe('clampScore', () => {
  it('keeps the score between 0 and 100', () => {
    expect(clampScore('85')).toBe(85);
    expect(clampScore('150')).toBe(100);
    expect(clampScore('-5')).toBe(0);
    expect(clampScore('')).toBe(0);
    expect(clampScore('abc')).toBe(0);
    expect(clampScore('1e3')).toBe(100);
    expect(clampScore('9e1')).toBe(90);
  });
});

describe('real import filters', () => {
  it('keeps a card without a score at any minimum', () => {
    const cards = [makeCard({ id: 'a', score: null }), makeCard({ id: 'b', score: 50 })];
    expect(filterCards(cards, DEFAULT_FILTERS).map(c => c.id)).toEqual(['a']);
  });
  it('hides a card without a target only when a target is selected', () => {
    const card = makeCard({ target: null });
    expect(filterCards([card], EMPTY_FILTERS)).toHaveLength(1);
    expect(filterCards([card], { ...EMPTY_FILTERS, targets: ['data'] })).toHaveLength(0);
  });
});
describe('import proof', () => { it('filterCards keeps a card without a score at any minimum', () => { expect(filterCards([makeCard({ score: null })], DEFAULT_FILTERS)).toHaveLength(1); }); });
