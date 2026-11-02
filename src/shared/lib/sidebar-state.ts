export const SIDEBAR_KEY = 'job-radar.sidebar-open';
export function readSidebarOpen(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_KEY) === 'true';
  } catch {
    return false;
  }
}
export function writeSidebarOpen(open: boolean): void {
  try {
    localStorage.setItem(SIDEBAR_KEY, String(open));
  } catch {
    // Storage unavailable: the in-memory state still works.
  }
}
