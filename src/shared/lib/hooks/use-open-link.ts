import { useCallback, useState } from 'react';

/** Opens a job link and remembers whether the last attempt failed, so the page can say so. */
export function useOpenLink(open: (id: string) => Promise<void>) {
  const [linkError, setLinkError] = useState(false);
  const openLink = useCallback(
    async (id: string) => {
      try {
        await open(id);
        setLinkError(false);
      } catch {
        setLinkError(true);
      }
    },
    [open],
  );
  return { openLink, linkError };
}
