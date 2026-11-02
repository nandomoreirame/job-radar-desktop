import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { MoveMenu } from "./MoveMenu";

describe("MoveMenu", () => {
  it("offers every other open stage", async () => {
    const onMove = vi.fn();
    render(<MoveMenu card={makeCard({ stage: "applied" })} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Staff Engineer" }));
    expect(await screen.findByRole("menuitem", { name: "Oferta" })).toBeInTheDocument();
    expect(screen.queryByRole("menuitem", { name: "Candidatado" })).not.toBeInTheDocument();
    expect(screen.queryByRole("menuitem", { name: "Encerradas" })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("menuitem", { name: "Oferta" }));
    expect(onMove).toHaveBeenCalledWith("offer", null);
  });

  it("hides the current outcome and closes with another one", async () => {
    const onMove = vi.fn();
    render(<MoveMenu card={makeCard({ stage: "closed", outcome: "rejected" })} onMove={onMove} />);
    await userEvent.click(screen.getByRole("button", { name: "Ações de Staff Engineer" }));
    expect(screen.queryByRole("menuitem", { name: "Recusada" })).not.toBeInTheDocument();
    await userEvent.click(await screen.findByRole("menuitem", { name: "Sem resposta" }));
    expect(onMove).toHaveBeenCalledWith("closed", "no_response");
  });
});
