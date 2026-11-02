import { useEffect } from "react";
export function useHotkey(key: string, handler: () => void): void {
  useEffect(() => { const onKeyDown = (event: KeyboardEvent) => { if (event.key.toLowerCase() !== key || !(event.ctrlKey || event.metaKey)) return; event.preventDefault(); handler(); }; window.addEventListener("keydown", onKeyDown); return () => window.removeEventListener("keydown", onKeyDown); }, [key, handler]);
}
