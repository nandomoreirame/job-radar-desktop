import { Archive, FileText, LayoutGrid, NotebookPen, Play, Settings, UserRound, type LucideIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { PageId } from '@/shared/config/navigation';
import {
  Sidebar as BaseSidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/shared/ui/sidebar';
type NavItem = { id: PageId; Icon: LucideIcon; pending?: number };
const NAV: NavItem[] = [
  { id: 'board', Icon: LayoutGrid },
  { id: 'discarded', Icon: Archive },
  { id: 'profile', Icon: UserRound },
  { id: 'resumes', Icon: FileText, pending: 3 },
  { id: 'prep', Icon: NotebookPen, pending: 2 },
  { id: 'settings', Icon: Settings },
];
const CRAWLER_STATUS = { time: '08:00', total: 142, fresh: 9, duration: 38, next: '12:00', ok: 8 };
type Props = { active: PageId; onNavigate: (page: PageId) => void };
export function AppSidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  const [notice, setNotice] = useState(false);
  return (
    <BaseSidebar collapsible="icon" className="!top-10 !h-[calc(100svh-2.5rem)]">
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <nav aria-label={t('nav.label')}>
              <SidebarMenu>
                {NAV.map(({ id, Icon, pending }) => (
                  <SidebarMenuItem key={id}>
                    <SidebarMenuButton
                      isActive={active === id}
                      aria-current={active === id ? 'page' : undefined}
                      tooltip={t(`nav.${id}`)}
                      onClick={() => onNavigate(id)}
                    >
                      <Icon aria-hidden="true" />
                      <span>{t(`nav.${id}`)}</span>
                    </SidebarMenuButton>
                    {pending !== undefined && (
                      <SidebarMenuBadge className="font-mono text-noir-amber">
                        <span aria-hidden="true">{pending}</span>
                        <span className="sr-only">{t('nav.pending', { count: pending })}</span>
                      </SidebarMenuBadge>
                    )}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </nav>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <section
          aria-label={t('crawler.title')}
          className="flex flex-col gap-1 rounded-lg border border-noir-border bg-noir-card/70 p-3 text-xs text-noir-muted group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:items-center group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
        >
          <p className="flex items-center gap-2 text-noir-teal">
            <span aria-hidden="true" className="size-2 shrink-0 rounded-full bg-noir-teal" />
            <span className="group-data-[collapsible=icon]:sr-only">{t('crawler.active')}</span>
          </p>
          <div className="flex flex-col gap-1 group-data-[collapsible=icon]:hidden">
            <p>{t('crawler.lastRun', CRAWLER_STATUS)}</p>
            <p>{t('crawler.duration', CRAWLER_STATUS)}</p>
            <p>{t('crawler.nextRun', CRAWLER_STATUS)}</p>
            <p>{t('crawler.sources', CRAWLER_STATUS)}</p>
          </div>
          <button
            type="button"
            title={t('crawler.runNow')}
            onClick={() => setNotice(true)}
            className="mt-2 flex items-center gap-2 rounded-md border border-noir-border px-2 py-1.5 text-left text-noir-text hover:border-noir-amber hover:text-noir-amber focus-visible:outline-2 focus-visible:outline-noir-amber group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:p-1"
          >
            <Play aria-hidden="true" className="size-3 shrink-0" />
            <span className="group-data-[collapsible=icon]:sr-only">{t('crawler.runNow')}</span>
          </button>
          {/* The crawler runs in the homelab for now; running it from the app is a later plan. */}
          {notice && (
            <p role="status" className="group-data-[collapsible=icon]:sr-only">
              {t('app.notImplemented')}
            </p>
          )}
        </section>
      </SidebarFooter>
    </BaseSidebar>
  );
}
