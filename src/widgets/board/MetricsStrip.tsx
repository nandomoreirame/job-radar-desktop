import { useTranslation } from "react-i18next";
import type { BoardMetrics } from "./model/metrics";

export function MetricsStrip({ metrics }: { metrics: BoardMetrics }) {
  const { t } = useTranslation();
  const none = t("metrics.none");
  const items = [
    { label: t("metrics.appliedWeek"), value: String(metrics.appliedThisWeek) },
    { label: t("metrics.responseRate"), value: metrics.responseRate === null ? none : `${metrics.responseRate}%` },
    { label: t("metrics.interviews"), value: String(metrics.interviewing) },
    {
      label: t("metrics.avgResponse"),
      value: metrics.avgDaysToResponse === null ? none : t("metrics.days", { value: metrics.avgDaysToResponse }),
    },
  ];
  return (
    <dl className="flex flex-wrap gap-x-8 gap-y-1 text-sm">
      {items.map(({ label, value }) => (
        <div key={label} className="flex gap-2">
          <dt className="text-noir-muted">{label}</dt>
          <dd className="font-mono">{value}</dd>
        </div>
      ))}
    </dl>
  );
}
