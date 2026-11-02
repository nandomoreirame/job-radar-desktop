import { useCallback, useEffect, useState } from "react";
import type { BoardApi } from "@/shared/api/board";
import type { BoardCard } from "@/shared/bindings/BoardCard";
import type { Outcome } from "@/shared/bindings/Outcome";
import type { Stage } from "@/shared/bindings/Stage";

export type BoardStatus = "loading" | "ready" | "error";

export function useBoard(api: BoardApi) {
  const [cards, setCards] = useState<BoardCard[]>([]);
  const [status, setStatus] = useState<BoardStatus>("loading");
  const [moveError, setMoveError] = useState(false);

  useEffect(() => {
    let alive = true;
    api.list().then(
      (list) => {
        if (!alive) return;
        setCards(list);
        setStatus("ready");
      },
      () => {
        if (alive) setStatus("error");
      },
    );
    return () => {
      alive = false;
    };
  }, [api]);

  const move = useCallback(
    async (id: string, stage: Stage, outcome: Outcome | null) => {
      try {
        const updated = await api.move(id, stage, outcome);
        setCards((list) => list.map((c) => (c.id === id ? updated : c)));
        setMoveError(false);
      } catch {
        // The store rejected the move; the card stays where it was.
        setMoveError(true);
        setCards((list) => [...list]);
      }
    },
    [api],
  );

  return { cards, status, moveError, move };
}
