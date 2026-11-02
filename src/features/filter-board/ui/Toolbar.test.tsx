import { render, screen } from "@testing-library/react";
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
});
