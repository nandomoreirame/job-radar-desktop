import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it } from "vitest";
import { expectNoAxeViolations } from "@/test/axe";
import { fakeApi, makeCard } from "@/test/factories";
import { BoardPage } from "./BoardPage";

const cards = [
  makeCard({ id: "a", title: "Alpha Engineer", unread: true }),
  makeCard({ id: "b", title: "Beta Lead", stage: "triage", aiRunning: "claude" }),
  makeCard({ id: "c", title: "Gamma Architect", stage: "in_process", processStep: "Técnica", salary: { pj: 34_000, clt: 26_000 } }),
  makeCard({ id: "d", title: "Delta Engineer", stage: "closed", outcome: "rejected", salary: { pj: 25_000, clt: null } }),
];

describe("BoardPage", () => {
  it("has no axe violations", async () => {
    const { container } = render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { level: 2, name: "Caixa de entrada" });
    await expectNoAxeViolations(container);
  });
  it("has no axe violations with the palette open", async () => {
    render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { level: 2, name: "Caixa de entrada" });
    await userEvent.keyboard("{Control>}k{/Control}");
    await screen.findByRole("combobox");
    await expectNoAxeViolations(document.body);
  });

});
