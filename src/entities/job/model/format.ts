import type { Salary } from "@/shared/bindings/Salary";

export function formatThousands(value: number): string {
  return String(Math.round(value / 1000));
}

export type ScoreTone = "high" | "good" | "low";

export function scoreTone(score: number): ScoreTone {
  if (score >= 90) return "high";
  if (score >= 80) return "good";
  return "low";
}

export function hasNegotiatedSalary(salary: Salary | null): salary is Salary {
  return salary !== null && (salary.pj !== null || salary.clt !== null);
}
