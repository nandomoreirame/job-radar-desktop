import { useCallback, useEffect, useState } from 'react';
import type { DiscardedApi } from '@/shared/api/board';
import type { DiscardedJob } from '@/shared/bindings/DiscardedJob';
export function useDiscarded(api: DiscardedApi) {
  const [jobs, setJobs] = useState<DiscardedJob[]>([]);
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [actionError, setActionError] = useState(false);
  useEffect(() => {
    let alive = true;
    api.list().then(
      list => {
        if (alive) {
          setJobs(list);
          setStatus('ready');
        }
      },
      () => alive && setStatus('error'),
    );
    return () => {
      alive = false;
    };
  }, [api]);
  const settle = useCallback(async (id: string, action: Promise<unknown>) => {
    try {
      await action;
      setJobs(list => list.filter(job => job.id !== id));
      setActionError(false);
    } catch {
      setActionError(true);
    }
  }, []);
  return {
    jobs,
    status,
    actionError,
    restore: (id: string) => settle(id, api.restore(id)),
    remove: (id: string) => settle(id, api.remove(id)),
  };
}
