import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { BoardApi } from "@/shared/api/board";

export function makeCard(overrides: Partial<BoardCard> = {}): BoardCard {
  return {
    id: "c01",
    title: "Staff Engineer",
    company: "Lumen Labs",
    source: "Greenhouse",
    target: "ai_llm",
    workModel: "remote",
    location: null,
    score: 94,
    stage: "inbox",
    outcome: null,
    processStep: null,
    salary: null,
    artifacts: { cv: false, coverage: false, interviewPrep: false },
    lastActivity: null,
    nextAction: null,
    aiRunning: null,
    unread: false,
    timeline: [],
    nextActionDate: null,
    recruiterName: null,
    recruiterEmail: null,
    recruiterLinkedin: null,
    jobUrl: null,
    applicationUrl: null,
    notes: '',
    ...overrides,
  };
}

/** In-memory BoardApi that applies moves like the Rust MockStore. */
export function fakeApi(cards: BoardCard[]): BoardApi {
  let state = [...cards];
  return {
    list: async () => state,
    move: async (id, stage, outcome) => {
      const card = state.find((c) => c.id === id);
      if (!card) throw new Error(`card not found: ${id}`);
      const moved = { ...card, stage, outcome };
      state = state.map((c) => (c.id === id ? moved : c));
      return moved;
    },
  };
}
