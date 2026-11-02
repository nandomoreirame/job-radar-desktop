import { useTranslation } from "react-i18next";
import type { PageId } from "@/shared/config/navigation";

export function PlaceholderPage({ page }: { page: Exclude<PageId, "board"> }) {
  const { t } = useTranslation();
  return (
    <section className="noir-panel m-6 rounded-xl p-8">
      <h1 className="text-xl font-semibold">{t(`nav.${page}`)}</h1>
      <p className="mt-2 text-noir-muted">{t("placeholder.body")}</p>
    </section>
  );
}
