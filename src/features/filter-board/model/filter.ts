import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Target } from "@/shared/bindings/Target";
export type BoardFilters = { sources: string[]; targets: Target[]; minScore: number; unreadOnly: boolean };
export const EMPTY_FILTERS: BoardFilters = { sources: [], targets: [], minScore: 0, unreadOnly: false };
export const fold = (value: string) => value.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase("pt-BR");
export function matchesText(text: string, query: string): boolean { return fold(text).includes(fold(query.trim())); }
export function clampScore(raw: string): number { const value = Number.parseInt(raw, 10); return Number.isNaN(value) ? 0 : Math.min(100, Math.max(0, value)); }
export function filterCards(cards: BoardCard[], filters: BoardFilters): BoardCard[] {
  return cards.filter((card) => (filters.sources.length === 0 || filters.sources.includes(card.source)) && (filters.targets.length === 0 || filters.targets.includes(card.target)) && card.score >= filters.minScore && (!filters.unreadOnly || card.unread));
}
