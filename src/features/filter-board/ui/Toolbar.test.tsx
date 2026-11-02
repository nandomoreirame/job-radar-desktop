import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EMPTY_FILTERS } from "../model/filter";
import { Toolbar } from "./Toolbar";
const sources = ["Gupy", "Lever"];
describe("Toolbar", () => {
  it("toggles a source filter", async () => { const onChange = vi.fn(); render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />); await userEvent.click(screen.getByRole("combobox", { name: "Fonte" })); await userEvent.click(await screen.findByRole("option", { name: "Gupy" })); expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, sources: ["Gupy"] }); });
  it("toggles a target filter", async () => { const onChange = vi.fn(); render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />); await userEvent.click(screen.getByRole("combobox", { name: "Alvo" })); await userEvent.click(await screen.findByRole("option", { name: "Dados" })); expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, targets: ["data"] }); });
  it("clamps the minimum score", () => { const onChange = vi.fn(); render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />); fireEvent.change(screen.getByRole("spinbutton", { name: "Score mín." }), { target: { value: "150" } }); expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, minScore: 100 }); });
  it("filters unread only", async () => { const onChange = vi.fn(); render(<Toolbar filters={EMPTY_FILTERS} sources={sources} onChange={onChange} />); await userEvent.click(screen.getByRole("checkbox", { name: "Somente não lidas" })); expect(onChange).toHaveBeenLastCalledWith({ ...EMPTY_FILTERS, unreadOnly: true }); });
  it("summarizes the selection in the trigger", () => { render(<Toolbar filters={{ ...EMPTY_FILTERS, sources }} sources={sources} onChange={() => {}} />); expect(screen.getByRole("combobox", { name: "Fonte" })).toHaveTextContent("2 selecionados"); });
});
