import { Archive, FileText, LayoutGrid, NotebookPen, Settings, UserRound, type LucideIcon } from 'lucide-react';
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
import { CrawlerCard } from './CrawlerCard';
import { useCrawler } from './use-crawler';
type NavItem = { id: PageId; Icon: LucideIcon; pending?: number };
const NAV: NavItem[] = [
  { id: 'board', Icon: LayoutGrid },
  { id: 'discarded', Icon: Archive },
  { id: 'profile', Icon: UserRound },
  { id: 'resumes', Icon: FileText, pending: 3 },
  { id: 'prep', Icon: NotebookPen, pending: 2 },
  { id: 'settings', Icon: Settings },
];
type Props = { active: PageId; onNavigate: (page: PageId) => void };
export function AppSidebar({ active, onNavigate }: Props) {
  const { t } = useTranslation();
  const { start, ...crawler } = useCrawler();
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
        <CrawlerCard {...crawler} onRun={() => void start()} />
      </SidebarFooter>
    </BaseSidebar>
  );
}
