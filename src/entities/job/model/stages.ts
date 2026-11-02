import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import type { Target } from "@/shared/bindings/Target";

/** Board column order; mirrors `Stage::ALL` in src-tauri/src/domain.rs. */
export const STAGES: readonly Stage[] = ["inbox", "triage", "applied", "in_process", "offer", "closed"];
export const OUTCOMES: readonly Outcome[] = ["hired", "rejected", "discarded", "no_response"];
export const TARGETS: readonly Target[] = ["ai_llm", "full_stack", "tech_lead", "data"];

export const isStage = (value: unknown): value is Stage => STAGES.includes(value as Stage);
