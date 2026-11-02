import { BoardPage } from "@/pages/board/BoardPage";
import { PlaceholderPage } from "@/pages/placeholder/PlaceholderPage";
import { DiscardedPage } from "@/pages/discarded/DiscardedPage";
import { AppShell } from "@/widgets/app-shell/AppShell";

export function App() {
  return (
    <AppShell
      renderPage={(page) => (page === "board" ? <BoardPage /> : page === "discarded" ? <DiscardedPage /> : <PlaceholderPage page={page} />)}
    />
  );
}
