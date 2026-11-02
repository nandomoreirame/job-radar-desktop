import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EMPTY_FILTERS } from "../model/filter";
import { Toolbar } from "./Toolbar";

describe("Toolbar", () => {
  it("reports the search text", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={["Gupy", "Lever"]} onChange={onChange} />);
    await userEvent.type(screen.getByRole("searchbox", { name: "Buscar vaga, empresa…" }), "v");
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, query: "v" });
  });

  it("toggles a source filter", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={["Gupy", "Lever"]} onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Fonte: Todas" }));
    await userEvent.click(await screen.findByRole("menuitemcheckbox", { name: "Gupy" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, sources: ["Gupy"] });
  });

  it("toggles a target filter", async () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={[]} onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Alvo: Todas" }));
    await userEvent.click(await screen.findByRole("menuitemcheckbox", { name: "Tech Lead" }));
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, targets: ["tech_lead"] });
  });

  it("summarises the selected filters", () => {
    render(
      <Toolbar
        filters={{ ...EMPTY_FILTERS, sources: ["Gupy"], targets: ["data", "ai_llm"] }}
        sources={["Gupy"]}
        onChange={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: /Fonte: 1 selecionado/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Alvo: 2 selecionados/ })).toBeInTheDocument();
  });

  it("reports the minimum score", () => {
    const onChange = vi.fn();
    render(<Toolbar filters={EMPTY_FILTERS} sources={[]} onChange={onChange} />);
    fireEvent.change(screen.getByRole("slider", { name: /Score mín\./ }), { target: { value: "75" } });
    expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, minScore: 75 });
  });
});
