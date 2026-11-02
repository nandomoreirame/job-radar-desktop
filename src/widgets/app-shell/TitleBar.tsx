import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";

export type WindowControls = {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
};

type Props = { controls?: WindowControls };

export function TitleBar({ controls }: Props) {
  const { t } = useTranslation();
  // Resolved on click: getCurrentWindow() needs the Tauri runtime.
  const run = (action: keyof WindowControls) => () => {
    void (controls ?? getCurrentWindow())[action]();
  };
  const buttons = [
    { action: "minimize", label: t("titlebar.minimize"), Icon: Minus },
    { action: "toggleMaximize", label: t("titlebar.maximize"), Icon: Square },
    { action: "close", label: t("titlebar.close"), Icon: X },
  ] as const;

  return (
    <header
      data-tauri-drag-region
      className="flex h-10 shrink-0 items-center justify-between border-b border-noir-border bg-noir-sidebar pl-4"
    >
      <span data-tauri-drag-region className="flex items-center gap-2 text-sm text-noir-muted">
        <span aria-hidden="true" className="size-2 rounded-full bg-noir-amber" />
        {t("app.name")}
      </span>
      <div className="flex h-full">
        {buttons.map(({ action, label, Icon }) => (
          <button
            key={action}
            type="button"
            aria-label={label}
            onClick={run(action)}
            className="grid h-full w-12 place-items-center text-noir-muted hover:bg-white/5 hover:text-noir-text focus-visible:bg-white/10 focus-visible:outline-none"
          >
            <Icon aria-hidden="true" className="size-3.5" />
          </button>
        ))}
      </div>
    </header>
  );
}
