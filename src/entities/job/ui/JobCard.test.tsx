import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { makeCard } from "@/test/factories";
import { JobCard } from "./JobCard";

describe("JobCard", () => {
  it("shows the job essentials", () => {
    render(<JobCard card={makeCard()} onOpen={() => {}} />);
    expect(screen.getByRole("heading", { name: /Staff Engineer/ })).toBeInTheDocument();
    expect(screen.getByText("Lumen Labs")).toBeInTheDocument();
    expect(screen.getByText("Greenhouse")).toBeInTheDocument();
    expect(screen.getByText("AI/LLM")).toBeInTheDocument();
    expect(screen.getByText("Remoto")).toBeInTheDocument();
    // The score pill reads "Score 94" for screen readers (sr-only prefix).
    expect(screen.getByText("Score 94")).toBeInTheDocument();
  });

  it("shows PJ and CLT when both were negotiated", () => {
    render(<JobCard card={makeCard({ stage: "applied", salary: { pj: 34_000, clt: 26_000 } })} onOpen={() => {}} />);
    expect(screen.getByText("PJ")).toBeInTheDocument();
    expect(screen.getByText("R$ 34k")).toBeInTheDocument();
    expect(screen.getByText("CLT")).toBeInTheDocument();
    expect(screen.getByText("R$ 26k")).toBeInTheDocument();
  });

  it("shows only PJ when CLT was not negotiated", () => {
    render(<JobCard card={makeCard({ stage: "applied", salary: { pj: 30_000, clt: null } })} onOpen={() => {}} />);
    expect(screen.getByText("R$ 30k")).toBeInTheDocument();
    expect(screen.queryByText("CLT")).not.toBeInTheDocument();
  });

  it("hides the salary when nothing was negotiated", () => {
    render(<JobCard card={makeCard({ salary: { pj: null, clt: null } })} onOpen={() => {}} />);
    expect(screen.queryByText("PJ")).not.toBeInTheDocument();
    expect(screen.queryByText(/R\$/)).not.toBeInTheDocument();
  });

  it("shows the outcome of a closed card", () => {
    render(<JobCard card={makeCard({ stage: "closed", outcome: "hired" })} onOpen={() => {}} />);
    expect(screen.getByText("Contratado")).toBeInTheDocument();
  });

  it("maps the outcome to a status variant", () => {
    render(<JobCard card={makeCard({ stage: "closed", outcome: "rejected" })} onOpen={() => {}} />);
    expect(screen.getByText("Recusada").closest('[data-slot="status"]')).toHaveAttribute("data-variant", "error");
  });

  it("shows the AI running status", () => {
    render(<JobCard card={makeCard({ aiRunning: "claude" })} onOpen={() => {}} />);
    expect(screen.getByRole("status")).toHaveTextContent("Claude trabalhando…");
  });

  it("opens the details when the title is activated", async () => {
    const onOpen = vi.fn();
    const card = makeCard();
    render(<JobCard card={card} onOpen={onOpen} />);
    await userEvent.click(screen.getByRole("button", { name: "Abrir detalhes de Staff Engineer" }));
    expect(onOpen).toHaveBeenCalledWith(card);
  });

  it("shows location, process step, unread flag and the next action", () => {
    render(
      <JobCard
        card={makeCard({ location: "São Paulo", processStep: "Técnica", unread: true, nextAction: "Responder recrutador" })}
        onOpen={() => {}}
      />,
    );
    expect(screen.getByText(/São Paulo/)).toBeInTheDocument();
    expect(screen.getByText("Técnica")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Não lida" })).toBeInTheDocument();
    expect(screen.getByText("Responder recrutador")).toBeInTheDocument();
  });
});
