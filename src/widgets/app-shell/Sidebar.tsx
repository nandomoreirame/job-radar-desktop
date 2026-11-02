import { FileText, LayoutGrid, NotebookPen, PanelLeft, Play, Settings, UserRound, type LucideIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { PageId } from '@/shared/config/navigation';
import {
  Sidebar as BaseSidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/shared/ui/sidebar';
type NavItem = { id: PageId; Icon: LucideIcon; pending?: number };
const NAV: NavItem[] = [
  { id: 'board', Icon: LayoutGrid },
  { id: 'profile', Icon: UserRound },
  { id: 'resumes', Icon: FileText, pending: 3 },
  { id: 'prep', Icon: NotebookPen, pending: 2 },
  { id: 'settings', Icon: Settings },
];
const CRAWLER_STATUS = { time: '08:00', total: 142, fresh: 9, duration: 38, next: '12:00', ok: 8 };
type Props = { active: PageId; onNavigate: (page: PageId) => void };
export function AppSidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  const { state, toggleSidebar } = useSidebar();
  return (
    <BaseSidebar collapsible="icon" className="!top-10 !h-[calc(100svh-2.5rem)]">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              onClick={toggleSidebar}
              tooltip={t('sidebar.toggle')}
              aria-expanded={state === 'expanded'}
            >
              <PanelLeft aria-hidden="true" />
              <span>{t('sidebar.toggle')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
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
      <SidebarFooter className="gap-1 text-xs text-noir-muted">
        <p className="flex items-center gap-2 px-2 text-noir-teal">
          <span aria-hidden="true" className="size-2 shrink-0 rounded-full bg-noir-teal" />
          <span className="group-data-[collapsible=icon]:sr-only">{t('crawler.active')}</span>
        </p>
        <p className="px-2 group-data-[collapsible=icon]:hidden">{t('crawler.lastRun', CRAWLER_STATUS)}</p>
        <p className="px-2 group-data-[collapsible=icon]:hidden">{t('crawler.duration', CRAWLER_STATUS)}</p>
        <p className="px-2 group-data-[collapsible=icon]:hidden">{t('crawler.nextRun', CRAWLER_STATUS)}</p>
        <p className="px-2 group-data-[collapsible=icon]:hidden">{t('crawler.sources', CRAWLER_STATUS)}</p>
        <button
          type="button"
          disabled
          title={t('crawler.runNow')}
          className="mt-2 flex items-center gap-2 rounded-md px-2 py-1 text-left text-noir-muted"
        >
          <Play aria-hidden="true" className="size-3" />
          <span className="group-data-[collapsible=icon]:sr-only">{t('crawler.runNow')}</span>
        </button>
      </SidebarFooter>
    </BaseSidebar>
  );
}
