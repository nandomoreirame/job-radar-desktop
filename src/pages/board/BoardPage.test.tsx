import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { fakeApi, makeCard } from "@/test/factories";
import { BoardPage } from "./BoardPage";

const cards = [
  makeCard({ id: "v", title: "Lead AI Architect", company: "Vértice Saúde" }),
  makeCard({ id: "o", title: "Tech Lead", company: "Orbita Pay" }),
];

describe("BoardPage", () => {
  it("filters the board from the search box", async () => {
    render(<BoardPage api={fakeApi(cards)} now={new Date("2026-11-02T12:00:00Z")} />);
    await screen.findByRole("heading", { name: /Lead AI Architect/ });
    await userEvent.type(screen.getByRole("searchbox"), "vertice");
    expect(screen.getByRole("heading", { name: /Lead AI Architect/ })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: /Tech Lead/ })).not.toBeInTheDocument();
  });
});
