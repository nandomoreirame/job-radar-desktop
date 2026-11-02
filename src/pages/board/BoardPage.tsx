import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { EMPTY_FILTERS, filterCards, type BoardFilters } from "@/features/filter-board/model/filter";
import { Toolbar } from "@/features/filter-board/ui/Toolbar";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { Board } from "@/widgets/board/Board";
import { MetricsStrip } from "@/widgets/board/MetricsStrip";
import { computeMetrics } from "@/widgets/board/model/metrics";

type Props = { api?: BoardApi; now?: Date };

export function BoardPage({ api = boardApi, now }: Props) {
  const { t } = useTranslation();
  const { cards, status } = useBoard(api);
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_FILTERS);
  const visible = useMemo(() => filterCards(cards, filters), [cards, filters]);
  // Metrics follow the filters, so a source or target can be judged on its own.
  const metrics = useMemo(() => computeMetrics(visible, now ?? new Date()), [visible, now]);
  const sources = useMemo(() => [...new Set(cards.map((card) => card.source))].sort(), [cards]);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <div className="flex flex-wrap items-center gap-4">
        <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
        <Toolbar filters={filters} sources={sources} onChange={setFilters} />
      </div>
      <MetricsStrip metrics={metrics} />
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && <Board cards={visible} onOpen={() => {}} />}
    </div>
  );
}
