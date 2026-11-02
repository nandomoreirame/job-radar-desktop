import { isStage } from "@/entities/job/model/stages";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Stage } from "@/shared/bindings/Stage";

/** What a drop means: move to a stage, close (outcome pending), or nothing. */
export type Drop = { kind: "move"; stage: Stage } | { kind: "close" } | null;

/**
 * Maps a drag end onto the intent it represents. Dropping on the card's own
 * column, outside a column, or with a stale id is a no-op.
 */
export function resolveDrop(card: BoardCard | undefined, overId: unknown): Drop {
  if (!card || !isStage(overId) || overId === card.stage) return null;
  return overId === "closed" ? { kind: "close" } : { kind: "move", stage: overId };
}
