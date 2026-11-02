import { useState, type ReactNode } from "react";
import type { PageId } from "@/shared/config/navigation";
import { Sidebar } from "./Sidebar";
import { TitleBar, type WindowControls } from "./TitleBar";

type Props = {
  renderPage: (page: PageId) => ReactNode;
  controls?: WindowControls;
};

export function AppShell({ renderPage, controls }: Props) {
  const [page, setPage] = useState<PageId>("board");
  return (
    <div className="noir-backdrop flex h-full flex-col">
      <TitleBar controls={controls} />
      <div className="flex min-h-0 flex-1">
        <Sidebar active={page} onNavigate={setPage} />
        <main className="flex min-w-0 flex-1 flex-col">{renderPage(page)}</main>
      </div>
    </div>
  );
}
