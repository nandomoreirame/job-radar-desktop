import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { Board } from "./Board";

const cards = [
  makeCard({ id: "a", title: "Alpha Engineer" }),
  makeCard({ id: "b", title: "Beta Engineer" }),
  makeCard({ id: "c", title: "Gamma Lead", stage: "offer" }),
];
const noop = () => {};

describe("Board", () => {
  it("renders the six columns in board order", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual([
      "Caixa de entrada",
      "Triagem",
      "Candidatado",
      "Em processo",
      "Oferta",
      "Encerradas",
    ]);
  });

  it("announces each column count", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    const inbox = screen.getByRole("region", { name: "Caixa de entrada" });
    expect(within(inbox).getByText("2 vagas")).toBeInTheDocument();
  });

  it("shows the empty state", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    const triage = screen.getByRole("region", { name: "Triagem" });
    expect(within(triage).getByText("Nenhuma vaga aqui.")).toBeInTheDocument();
  });

  it("moves a card through the Mover para menu", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Alpha Engineer" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Triagem" }));
    expect(onMove).toHaveBeenCalledWith("a", "triage", null);
  });

  it("closes a card with the chosen outcome", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Alpha Engineer" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Contratado" }));
    expect(onMove).toHaveBeenCalledWith("a", "closed", "hired");
  });
  it("labels each drag handle", () => {
    render(<Board cards={cards} onOpen={noop} onMove={noop} />);
    expect(screen.getByRole("button", { name: "Arrastar Alpha Engineer" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Arrastar Gamma Lead" })).toBeInTheDocument();
  });

  it("keeps the move menu next to the handle", async () => {
    const onMove = vi.fn();
    render(<Board cards={cards} onOpen={noop} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Gamma Lead" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Em processo" }));
    expect(onMove).toHaveBeenCalledWith("c", "in_process", null);
  });

});
