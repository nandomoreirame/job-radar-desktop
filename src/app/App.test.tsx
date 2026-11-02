import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => []) }));

describe("App", () => {
  it("shows a placeholder for pages not built yet", async () => {
    render(<App />);
    await userEvent.click(screen.getByRole("button", { name: /Meus currículos/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Meus currículos" })).toBeInTheDocument();
    expect(screen.getByText("Esta tela chega nos próximos planos.")).toBeInTheDocument();
  });
});
