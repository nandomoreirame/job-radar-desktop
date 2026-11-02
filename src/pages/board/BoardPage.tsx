import { useTranslation } from "react-i18next";
import { useBoard } from "@/entities/job/model/use-board";
import { boardApi, type BoardApi } from "@/shared/api/board";
import { Board } from "@/widgets/board/Board";

type Props = { api?: BoardApi };

export function BoardPage({ api = boardApi }: Props) {
  const { t } = useTranslation();
  const { cards, status } = useBoard(api);
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-6">
      <h1 className="text-2xl font-semibold">{t("board.title")}</h1>
      {status === "loading" && <p role="status">{t("board.loading")}</p>}
      {status === "error" && <p role="alert">{t("board.error")}</p>}
      {status === "ready" && <Board cards={cards} onOpen={() => {}} />}
    </div>
  );
}
