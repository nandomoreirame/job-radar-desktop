import { describe, expect, it } from "vitest";
import i18n from "@/app/i18n";
import en from "./en.json";
import ptBR from "./pt-BR.json";

function keys(tree: object, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === "object" && value !== null
      ? keys(value as object, `${prefix}${key}.`)
      : [`${prefix}${key}`],
  );
}

describe("locales", () => {
  it("have the same keys in pt-BR and en", () => {
    expect(keys(en).sort()).toEqual(keys(ptBR).sort());
  });

  it("default to pt-BR", () => {
    expect(i18n.language).toBe("pt-BR");
    expect(i18n.t("stage.inbox")).toBe("Caixa de entrada");
  });

  it("translate to English", () => {
    expect(i18n.getFixedT("en")("stage.inbox")).toBe("Inbox");
  });
});
