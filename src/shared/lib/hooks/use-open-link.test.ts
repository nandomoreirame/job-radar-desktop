import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useOpenLink } from './use-open-link';

describe('useOpenLink', () => {
  it('flags a link that fails to open and clears it on success', async () => {
    const open = vi.fn().mockRejectedValueOnce('open_failed').mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useOpenLink(open));
    await act(() => result.current.openLink('c01'));
    expect(open).toHaveBeenCalledWith('c01');
    expect(result.current.linkError).toBe(true);
    await act(() => result.current.openLink('c01'));
    expect(result.current.linkError).toBe(false);
  });
});
