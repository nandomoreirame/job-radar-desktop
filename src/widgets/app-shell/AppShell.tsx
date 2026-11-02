import { useState, type ReactNode } from "react";
import type { PageId } from "@/shared/config/navigation";
import { readSidebarOpen, writeSidebarOpen } from "@/shared/lib/sidebar-state";
import { SidebarInset, SidebarProvider } from "@/shared/ui/sidebar";
import { AppSidebar } from "./Sidebar";
import { TitleBar, type WindowControls } from "./TitleBar";
type Props = { renderPage: (page: PageId) => ReactNode; controls?: WindowControls };
export function AppShell({ renderPage, controls }: Props) { const [page, setPage] = useState<PageId>("board"); const [sidebarOpen, setSidebarOpen] = useState(readSidebarOpen); const handleSidebar = (open: boolean) => { setSidebarOpen(open); writeSidebarOpen(open); }; return <div className="noir-backdrop flex h-full flex-col"><TitleBar controls={controls} /><SidebarProvider open={sidebarOpen} onOpenChange={handleSidebar} className="min-h-0 flex-1"><AppSidebar active={page} onNavigate={setPage} /><SidebarInset className="min-w-0 bg-transparent">{renderPage(page)}</SidebarInset></SidebarProvider></div>; }
