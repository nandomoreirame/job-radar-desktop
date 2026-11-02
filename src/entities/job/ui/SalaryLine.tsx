import { useTranslation } from "react-i18next";
import type { Salary } from "@/shared/bindings/Salary";
import { formatThousands, hasNegotiatedSalary } from "../model/format";

export function SalaryLine({ salary }: { salary: Salary | null }) {
  const { t } = useTranslation();
  if (!hasNegotiatedSalary(salary)) return null;
  const regimes = [
    { regime: "pj", value: salary.pj },
    { regime: "clt", value: salary.clt },
  ] as const;
  return (
    <p className="flex gap-4 text-sm">
      {regimes.map(({ regime, value }) =>
        value === null ? null : (
          <span key={regime} className="flex gap-1.5">
            <span className="text-noir-muted">{t(`salary.${regime}`)}</span>
            <span className="font-mono">{t("salary.value", { value: formatThousands(value) })}</span>
          </span>
        ),
      )}
    </p>
  );
}
