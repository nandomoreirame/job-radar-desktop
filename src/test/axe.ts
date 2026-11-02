import axe from "axe-core";
import { expect } from "vitest";

/**
 * WCAG 2.1 A/AA rules that jsdom can evaluate. Color contrast is off because
 * jsdom does not compute styles; it is checked by hand against the Noir tokens.
 */
export async function expectNoAxeViolations(container: Element): Promise<void> {
  const results = await axe.run(container, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] },
    rules: { "color-contrast": { enabled: false } },
  });
  const report = results.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
  expect(report).toEqual([]);
}
