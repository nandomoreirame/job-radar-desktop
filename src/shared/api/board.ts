import { invoke } from "@tauri-apps/api/core";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";

export type BoardApi = {
  list(): Promise<BoardCard[]>;
  move(id: string, stage: Stage, outcome: Outcome | null): Promise<BoardCard>;
};

/** Thin wrappers over the Rust commands in `src-tauri/src/commands.rs`. */
export const boardApi: BoardApi = {
  list: () => invoke<BoardCard[]>("list_board"),
  move: (id, stage, outcome) => invoke<BoardCard>("move_card", { id, stage, outcome }),
};
