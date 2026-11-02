import { invoke } from "@tauri-apps/api/core";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { CardDetails } from "@/shared/bindings/CardDetails";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";

export type BoardApi = {
  list(): Promise<BoardCard[]>;
  move(id: string, stage: Stage, outcome: Outcome | null): Promise<BoardCard>;
  /** Rejects with `<field>: <reason>` when the Rust side refuses a field. */
  update(id: string, details: CardDetails): Promise<BoardCard>;
};

/** Thin wrappers over the Rust commands in `src-tauri/src/commands.rs`. */
export const boardApi: BoardApi = {
  list: () => invoke<BoardCard[]>("list_board"),
  move: (id, stage, outcome) => invoke<BoardCard>("move_card", { id, stage, outcome }),
  update: (id, details) => invoke<BoardCard>("update_card", { id, details }),
};
