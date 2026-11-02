import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Target } from "@/shared/bindings/Target";

export type BoardFilters = {
  query: string;
  sources: string[];
  targets: Target[];
  minScore: number;
};

export const EMPTY_FILTERS: BoardFilters = { query: "", sources: [], targets: [], minScore: 0 };

/** Lowercase without diacritics, so "vertice" finds "Vértice". */
const fold = (value: string) => value.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase("pt-BR");

export function filterCards(cards: BoardCard[], filters: BoardFilters): BoardCard[] {
  const query = fold(filters.query.trim());
  return cards.filter(
    (card) =>
      (query === "" || fold(card.title).includes(query) || fold(card.company).includes(query)) &&
      (filters.sources.length === 0 || filters.sources.includes(card.source)) &&
      (filters.targets.length === 0 || filters.targets.includes(card.target)) &&
      card.score >= filters.minScore,
  );
}
