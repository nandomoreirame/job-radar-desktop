import { useCallback, useEffect, useRef, useState } from 'react';
import { CRAWLER_CHANGED, crawlerApi, type CrawlerApi } from '@/shared/api/board';
import type { CrawlRun } from '@/shared/bindings/CrawlRun';

/** How often the board reloads while a crawl adds cards. */
const RELOAD_EVERY_MS = 5000;

export function useCrawler(api: CrawlerApi = crawlerApi) {
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [run, setRun] = useState<CrawlRun | null>(null);
  const lastReload = useRef<number | null>(null);

  useEffect(() => {
    let alive = true;
    const refresh = () =>
      api.status().then(
        status => {
          if (!alive) return;
          setRunning(status.running);
          setRun(status.run);
        },
        // No answer leaves the card as it is; the next event tries again.
        () => {},
      );
    const tellBoard = () => window.dispatchEvent(new Event(CRAWLER_CHANGED));
    void refresh();
    // Without a backend to listen to, the card still works; there is just no live progress.
    const noSubscription = () => () => {};
    const subscriptions = [
      api
        .onProgress(text => {
          setRunning(true);
          setProgress(text);
          const now = Date.now();
          if (lastReload.current === null || now - lastReload.current >= RELOAD_EVERY_MS) {
            lastReload.current = now;
            tellBoard();
          }
        })
        .catch(noSubscription),
      api
        .onFinished(() => {
          setRunning(false);
          setProgress(null);
          lastReload.current = null;
          void refresh();
          tellBoard();
        })
        .catch(noSubscription),
    ];
    return () => {
      alive = false;
      for (const subscription of subscriptions) void subscription.then(off => off());
    };
  }, [api]);

  const start = useCallback(async () => {
    setError(null);
    setRunning(true);
    try {
      await api.start();
    } catch (refused) {
      setRunning(false);
      setError(String(refused));
    }
  }, [api]);

  return { running, progress, error, run, start };
}
