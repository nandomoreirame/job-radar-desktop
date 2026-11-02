import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { OutcomeDialog } from "./OutcomeDialog";

describe("OutcomeDialog", () => {
  it("returns the chosen outcome", async () => {
    const onPick = vi.fn();
    render(<OutcomeDialog open onPick={onPick} onCancel={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Recusada" }));
    expect(onPick).toHaveBeenCalledWith("rejected");
  });

  it("cancels without moving", async () => {
    const onPick = vi.fn();
    const onCancel = vi.fn();
    render(<OutcomeDialog open onPick={onPick} onCancel={onCancel} />);
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onPick).not.toHaveBeenCalled();
  });
});
