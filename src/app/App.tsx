import { useTranslation } from "react-i18next";
import { PlaceholderPage } from "@/pages/placeholder/PlaceholderPage";
import { AppShell } from "@/widgets/app-shell/AppShell";

function BoardSlot() {
  const { t } = useTranslation();
  return <h1 className="p-6 text-2xl font-semibold">{t("board.title")}</h1>;
}

export function App() {
  return (
    <AppShell
      renderPage={(page) => (page === "board" ? <BoardSlot /> : <PlaceholderPage page={page} />)}
    />
  );
}
