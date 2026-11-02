import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { JobDialog } from "./JobDialog";

const card = makeCard({
  stage: "applied",
  salary: { pj: 28_000, clt: null },
  timeline: [{ at: "2026-10-30", kind: "applied", label: "Candidatura enviada" }],
});

describe("JobDialog", () => {
  it("shows the details and the timeline", () => {
    render(<JobDialog card={card} onOpenChange={() => {}} />);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "Staff Engineer" })).toBeInTheDocument();
    expect(within(dialog).getByText("R$ 28k")).toBeInTheDocument();
    expect(within(dialog).getByText("Candidatura enviada")).toBeInTheDocument();
  });

  it("keeps CV generation disabled until it exists", () => {
    render(<JobDialog card={card} onOpenChange={() => {}} />);
    const button = screen.getByRole("button", { name: "Gerar CV customizado" });
    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Em breve");
  });

  it("closes with Escape", async () => {
    const onOpenChange = vi.fn();
    render(<JobDialog card={card} onOpenChange={onOpenChange} />);
    await userEvent.keyboard("{Escape}");
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("explains a missing salary and marks each artifact", () => {
    render(
      <JobDialog
        card={makeCard({ salary: null, artifacts: { cv: true, coverage: false, interviewPrep: true } })}
        onOpenChange={() => {}}
      />,
    );
    expect(screen.getByText("Ainda sem negociação.")).toBeInTheDocument();
    expect(screen.getByText(/CV customizado: pronto/)).toBeInTheDocument();
    expect(screen.getByText(/Revisão de cobertura: pendente/)).toBeInTheDocument();
  });
});
