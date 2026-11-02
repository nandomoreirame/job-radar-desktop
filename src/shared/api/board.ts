import { invoke } from "@tauri-apps/api/core";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { CardDetails } from "@/shared/bindings/CardDetails";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";
import type { DiscardedJob } from "@/shared/bindings/DiscardedJob";

export type BoardApi = {
  list(): Promise<BoardCard[]>;
  move(id: string, stage: Stage, outcome: Outcome | null): Promise<BoardCard>;
  /** Rejects with `<field>: <reason>` when the Rust side refuses a field. */
  update(id: string, details: CardDetails): Promise<BoardCard>;
  openLink(id: string): Promise<void>;
};
export type DiscardedApi = { list(): Promise<DiscardedJob[]>; restore(id: string): Promise<BoardCard>; remove(id: string): Promise<void>; openLink(id: string): Promise<void> };
export const discardedApi: DiscardedApi = { list: () => invoke<DiscardedJob[]>("list_discarded"), restore: id => invoke<BoardCard>("restore_card", { id }), remove: id => invoke<void>("delete_card", { id }), openLink: id => invoke<void>("open_job_url", { id }) };

/** Thin wrappers over the Rust commands in `src-tauri/src/commands.rs`. */
export const boardApi: BoardApi = {
  list: () => invoke<BoardCard[]>("list_board"),
  move: (id, stage, outcome) => invoke<BoardCard>("move_card", { id, stage, outcome }),
  update: (id, details) => invoke<BoardCard>("update_card", { id, details }),
  openLink: id => invoke<void>("open_job_url", { id }),
};
