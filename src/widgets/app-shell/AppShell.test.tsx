import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "./AppShell";

const controls = () => ({
  minimize: vi.fn(async () => {}),
  toggleMaximize: vi.fn(async () => {}),
  close: vi.fn(async () => {}),
});

describe("AppShell", () => {
  it("switches the page from the sidebar", async () => {
    render(<AppShell controls={controls()} renderPage={(page) => <h1>{page}</h1>} />);
    expect(screen.getByRole("heading", { name: "board" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Meu perfil/ }));
    expect(screen.getByRole("heading", { name: "profile" })).toBeInTheDocument();
  });

  it("marks the active item", async () => {
    render(<AppShell controls={controls()} renderPage={() => null} />);
    await userEvent.click(screen.getByRole("button", { name: /Configuração/ }));
    expect(screen.getByRole("button", { name: /Configuração/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("button", { name: /Board/ })).not.toHaveAttribute("aria-current");
  });

  it("drives the window controls", async () => {
    const window = controls();
    render(<AppShell controls={window} renderPage={() => null} />);
    await userEvent.click(screen.getByRole("button", { name: "Minimizar" }));
    await userEvent.click(screen.getByRole("button", { name: "Maximizar" }));
    await userEvent.click(screen.getByRole("button", { name: "Fechar" }));
    expect(window.minimize).toHaveBeenCalledOnce();
    expect(window.toggleMaximize).toHaveBeenCalledOnce();
    expect(window.close).toHaveBeenCalledOnce();
  });
});
