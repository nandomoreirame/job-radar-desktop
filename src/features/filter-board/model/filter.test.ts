import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { EMPTY_FILTERS, filterCards } from "./filter";

const v = makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde", source: "Gmail", target: "ai_llm", score: 91 });
const n = makeCard({ id: "n", title: "Engenheiro de Dados", company: "Nébula Data", source: "Lever", target: "data", score: 79 });
const o = makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay", source: "Gupy", target: "tech_lead", score: 82 });
const t = makeCard({ id: "t", title: "Análise de Dados", company: "Aurora Labs", source: "Lever", target: "data", score: 70 });
const ids = (cards: { id: string }[]) => cards.map((c) => c.id);

describe("filterCards", () => {
  it("keeps every card with empty filters", () => {
    expect(ids(filterCards([v, n, o], EMPTY_FILTERS))).toEqual(["v", "n", "o"]);
  });

  it("matches company names without accents", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, query: "vertice" }))).toEqual(["v"]);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, query: "SAUDE" }))).toEqual(["v"]);
    expect(ids(filterCards([v, n, o, t], { ...EMPTY_FILTERS, query: "analise" }))).toEqual(["t"]);
  });

  it("filters by source and target", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Gupy"], targets: ["tech_lead"] }))).toEqual(["o"]);
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, sources: ["Lever", "Gmail"] }))).toEqual(["v", "n"]);
  });

  it("drops cards below the minimum score", () => {
    expect(ids(filterCards([v, n, o], { ...EMPTY_FILTERS, minScore: 80 }))).toEqual(["v", "o"]);
  });
});
