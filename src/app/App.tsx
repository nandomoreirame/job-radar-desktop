import { BoardPage } from "@/pages/board/BoardPage";
import { PlaceholderPage } from "@/pages/placeholder/PlaceholderPage";
import { AppShell } from "@/widgets/app-shell/AppShell";

export function App() {
  return (
    <AppShell
      renderPage={(page) => (page === "board" ? <BoardPage /> : <PlaceholderPage page={page} />)}
    />
  );
}
