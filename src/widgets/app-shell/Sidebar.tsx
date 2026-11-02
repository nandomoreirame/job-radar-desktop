import { FileText, LayoutGrid, NotebookPen, Settings, UserRound, type LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { PageId } from "@/shared/config/navigation";
import { cn } from "@/shared/lib/utils";

type NavItem = { id: PageId; Icon: LucideIcon; pending?: number };

// Pending counts and crawler status are mocks until the store backs them.
const NAV: NavItem[] = [
  { id: "board", Icon: LayoutGrid },
  { id: "profile", Icon: UserRound },
  { id: "resumes", Icon: FileText, pending: 3 },
  { id: "prep", Icon: NotebookPen, pending: 2 },
  { id: "settings", Icon: Settings },
];
const CRAWLER_STATUS = { time: "08:00", total: 142, fresh: 9 };

type Props = { active: PageId; onNavigate: (page: PageId) => void };

export function Sidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-noir-border bg-noir-sidebar">
      <p className="flex items-center gap-2 px-5 py-5 text-lg font-semibold text-noir-amber">
        <span aria-hidden="true" className="size-2.5 rounded-full bg-noir-amber" />
        {t("app.name")}
      </p>
      <nav aria-label={t("nav.label")} className="flex-1 px-3">
        <ul className="flex flex-col gap-1">
          {NAV.map(({ id, Icon, pending }) => (
            <li key={id}>
              <button
                type="button"
                aria-current={active === id ? "page" : undefined}
                onClick={() => onNavigate(id)}
                className={cn(
                  "flex w-full items-center gap-3 rounded-md border-l-2 border-transparent px-3 py-2.5 text-left text-sm text-noir-muted hover:bg-white/5 hover:text-noir-text focus-visible:outline-2 focus-visible:outline-noir-amber",
                  active === id && "border-noir-amber bg-white/5 text-noir-text",
                )}
              >
                <Icon aria-hidden="true" className="size-4" />
                <span className="flex-1">{t(`nav.${id}`)}</span>
                {pending !== undefined && (
                  <span className="rounded-full bg-noir-amber/15 px-2 font-mono text-xs text-noir-amber">
                    <span aria-hidden="true">{pending}</span>
                    <span className="sr-only">{t("nav.pending", { count: pending })}</span>
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      </nav>
      <footer className="flex flex-col gap-1 border-t border-noir-border px-5 py-4 text-xs text-noir-muted">
        <p className="flex items-center gap-2 text-noir-teal">
          <span aria-hidden="true" className="size-2 rounded-full bg-noir-teal" />
          {t("crawler.active")}
        </p>
        <p>{t("crawler.lastRun", CRAWLER_STATUS)}</p>
        <p className="mt-2 font-serif italic">{t("app.signature")} •</p>
      </footer>
    </aside>
  );
}
