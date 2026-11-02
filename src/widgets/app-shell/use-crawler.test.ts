import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { CRAWLER_CHANGED, type CrawlerApi } from '@/shared/api/board';
import type { CrawlerStatus } from '@/shared/bindings/CrawlerStatus';
import { useCrawler } from './use-crawler';

const lastRun = { startedAt: 'a', finishedAt: 'b', stats: '{}' };

function fakeCrawler(status: CrawlerStatus = { running: false, run: null }) {
  const handlers: { progress?: (text: string) => void; finished?: () => void } = {};
  const offProgress = vi.fn();
  const offFinished = vi.fn();
  const api: CrawlerApi = {
    status: vi.fn(async () => status),
    start: vi.fn(async () => {}),
    onProgress: async handler => {
      handlers.progress = handler;
      return offProgress;
    },
    onFinished: async handler => {
      handlers.finished = handler;
      return offFinished;
    },
  };
  return { api, handlers, offProgress, offFinished };
}

afterEach(() => vi.useRealTimers());

describe('useCrawler', () => {
  it('loads the status on mount', async () => {
    const { api } = fakeCrawler({ running: true, run: lastRun });
    const { result } = renderHook(() => useCrawler(api));
    await waitFor(() => expect(result.current.run).toEqual(lastRun));
    expect(result.current.running).toBe(true);
  });

  it('keeps a refused start as the error and stops running', async () => {
    const { api } = fakeCrawler();
    api.start = vi.fn(async () => Promise.reject('config_missing'));
    const { result } = renderHook(() => useCrawler(api));
    await act(async () => {}); // lets the first status answer before the click
    await act(() => result.current.start());
    expect(result.current.error).toBe('config_missing');
    expect(result.current.running).toBe(false);
  });

  it('clears the old error on a new start', async () => {
    const { api } = fakeCrawler();
    api.start = vi.fn(async () => Promise.reject('config_missing'));
    const { result } = renderHook(() => useCrawler(api));
    await act(() => result.current.start());
    api.start = vi.fn(async () => {});
    await act(() => result.current.start());
    expect(result.current.error).toBeNull();
    expect(result.current.running).toBe(true);
  });

  it('refreshes and tells the board when the run finishes', async () => {
    const { api, handlers } = fakeCrawler();
    const changed = vi.fn();
    window.addEventListener(CRAWLER_CHANGED, changed);
    const { result } = renderHook(() => useCrawler(api));
    await waitFor(() => expect(handlers.finished).toBeDefined());
    act(() => handlers.progress?.('gupy: 1/2'));
    expect(result.current.progress).toBe('gupy: 1/2');
    api.status = vi.fn(async () => ({ running: false, run: lastRun }));
    act(() => handlers.finished?.());
    await waitFor(() => expect(result.current.run).toEqual(lastRun));
    expect(result.current.progress).toBeNull();
    expect(result.current.running).toBe(false);
    expect(changed).toHaveBeenCalled();
    window.removeEventListener(CRAWLER_CHANGED, changed);
  });

  it('refreshes the board at most once per 5 s while running', async () => {
    const { api, handlers } = fakeCrawler();
    const changed = vi.fn();
    window.addEventListener(CRAWLER_CHANGED, changed);
    const { result } = renderHook(() => useCrawler(api));
    await waitFor(() => expect(handlers.progress).toBeDefined());
    expect(result.current.running).toBe(false);
    const now = vi.spyOn(Date, 'now');
    now.mockReturnValue(100_000);
    act(() => handlers.progress?.('a'));
    act(() => handlers.progress?.('b'));
    expect(changed).toHaveBeenCalledTimes(1);
    now.mockReturnValue(104_999);
    act(() => handlers.progress?.('c'));
    expect(changed).toHaveBeenCalledTimes(1);
    now.mockReturnValue(105_000);
    act(() => handlers.progress?.('d'));
    expect(changed).toHaveBeenCalledTimes(2);
    now.mockRestore();
    window.removeEventListener(CRAWLER_CHANGED, changed);
  });

  it('unsubscribes on unmount', async () => {
    const { api, handlers, offProgress, offFinished } = fakeCrawler();
    const { unmount } = renderHook(() => useCrawler(api));
    await waitFor(() => expect(handlers.finished).toBeDefined());
    unmount();
    await waitFor(() => expect(offFinished).toHaveBeenCalledOnce());
    expect(offProgress).toHaveBeenCalledOnce();
  });

  it('survives a backend that cannot answer', async () => {
    const { api } = fakeCrawler();
    api.status = vi.fn(async () => Promise.reject(new Error('no ipc')));
    api.onProgress = async () => Promise.reject(new Error('no ipc'));
    api.onFinished = async () => Promise.reject(new Error('no ipc'));
    const { result } = renderHook(() => useCrawler(api));
    await waitFor(() => expect(api.status).toHaveBeenCalled());
    expect(result.current.run).toBeNull();
  });
});
