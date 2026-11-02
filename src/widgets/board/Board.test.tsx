import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { makeCard } from "@/test/factories";
import { Board } from "./Board";

const cards = [makeCard({ id: "a" }), makeCard({ id: "b" }), makeCard({ id: "c", stage: "offer" })];

describe("Board", () => {
  it("renders the six columns in board order", () => {
    render(<Board cards={cards} onOpen={() => {}} />);
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
    render(<Board cards={cards} onOpen={() => {}} />);
    const inbox = screen.getByRole("region", { name: "Caixa de entrada" });
    expect(within(inbox).getByText("2 vagas")).toBeInTheDocument();
  });

  it("shows the empty state", () => {
    render(<Board cards={cards} onOpen={() => {}} />);
    const triage = screen.getByRole("region", { name: "Triagem" });
    expect(within(triage).getByText("Nenhuma vaga aqui.")).toBeInTheDocument();
  });
});
