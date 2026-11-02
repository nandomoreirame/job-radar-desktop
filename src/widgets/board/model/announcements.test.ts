import type { Announcements } from "@dnd-kit/core";
import { describe, expect, it } from "vitest";
import i18n from "@/app/i18n";
import { makeCard } from "@/test/factories";
import { buildAnnouncements } from "./announcements";

const t = i18n.getFixedT("pt-BR");
const cards = [makeCard({ id: "a", title: "Alpha Engineer" })];
const announcements = buildAnnouncements(t, cards);

type Event = Parameters<NonNullable<Announcements["onDragEnd"]>>[0];
const event = (active: string, over?: string) =>
  ({ active: { id: active }, over: over ? { id: over } : null }) as unknown as Event;

describe("buildAnnouncements", () => {
  it("names the card picked up", () => {
    expect(announcements.onDragStart?.(event("a"))).toBe("Alpha Engineer selecionada.");
  });

  it("names the column hovered", () => {
    expect(announcements.onDragOver?.(event("a", "triage"))).toBe("Alpha Engineer sobre Triagem.");
  });

  it("stays quiet when hovering nothing", () => {
    expect(announcements.onDragOver?.(event("a"))).toBeUndefined();
  });

  it("reports where the card landed", () => {
    expect(announcements.onDragEnd?.(event("a", "offer"))).toBe("Alpha Engineer solta em Oferta.");
  });

  it("reports a cancelled drop", () => {
    expect(announcements.onDragEnd?.(event("a"))).toBe("Movimento de Alpha Engineer cancelado.");
    expect(announcements.onDragCancel?.(event("a"))).toBe("Movimento de Alpha Engineer cancelado.");
  });

  it("has no column name for unknown targets", () => {
    expect(announcements.onDragEnd?.(event("a", "bogus"))).toBe("Alpha Engineer solta em .");
  });
});
