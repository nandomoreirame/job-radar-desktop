import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/inter";
import "@fontsource/jetbrains-mono";
import "@/shared/styles/theme.css";
import "@/app/i18n";
import { App } from "@/app/App";
import { applyVisualMode, readStoredVisualMode, resolveVisualMode } from "@/shared/lib/visual-mode";

applyVisualMode(resolveVisualMode(readStoredVisualMode(), import.meta.env.VITE_VISUAL_MODE));

const root = document.getElementById("root");
if (!root) throw new Error("#root element missing from index.html");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
