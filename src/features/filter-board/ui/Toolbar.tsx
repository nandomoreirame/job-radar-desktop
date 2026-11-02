import { ChevronDown, Mail, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TARGETS } from "@/entities/job/model/stages";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";
import { Input } from "@/shared/ui/input";
import type { BoardFilters } from "../model/filter";

type Props = { filters: BoardFilters; sources: string[]; onChange: (filters: BoardFilters) => void };

const toggle = <T,>(list: T[], value: T): T[] =>
  list.includes(value) ? list.filter((item) => item !== value) : [...list, value];

export function Toolbar({ filters, sources, onChange }: Props) {
  const { t } = useTranslation();
  const summary = (count: number) => (count === 0 ? t("filters.all") : t("filters.selected", { count }));

  return (
    <div role="search" className="flex flex-1 flex-wrap items-center gap-3">
      <div className="relative w-[420px] max-w-full">
        <Search aria-hidden="true" className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-noir-muted" />
        <Input
          type="search"
          aria-label={t("filters.search")}
          placeholder={t("filters.search")}
          value={filters.query}
          onChange={(event) => onChange({ ...filters, query: event.target.value })}
          className="h-9 pl-9"
        />
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="h-9">
            {t("filters.source")}: {summary(filters.sources.length)}
            <ChevronDown aria-hidden="true" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {sources.map((source) => (
            <DropdownMenuCheckboxItem
              key={source}
              checked={filters.sources.includes(source)}
              onCheckedChange={() => onChange({ ...filters, sources: toggle(filters.sources, source) })}
              onSelect={(event) => event.preventDefault()}
            >
              {source}
            </DropdownMenuCheckboxItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" className="h-9">
            {t("filters.target")}: {summary(filters.targets.length)}
            <ChevronDown aria-hidden="true" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {TARGETS.map((target) => (
            <DropdownMenuCheckboxItem
              key={target}
              checked={filters.targets.includes(target)}
              onCheckedChange={() => onChange({ ...filters, targets: toggle(filters.targets, target) })}
              onSelect={(event) => event.preventDefault()}
            >
              {t(`target.${target}`)}
            </DropdownMenuCheckboxItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <label className="flex h-9 items-center gap-2 rounded-md border border-noir-border px-3 text-sm">
        <span>{t("filters.minScore")}</span>
        <input
          type="range"
          min={0}
          max={100}
          step={5}
          value={filters.minScore}
          onChange={(event) => onChange({ ...filters, minScore: Number(event.target.value) })}
          className="w-24 accent-noir-amber"
        />
        <span className="w-6 font-mono">{filters.minScore}</span>
      </label>

      {/* Gmail sync arrives with the AI CLI runner; disabled until then. */}
      <Button variant="outline" className="ml-auto h-9" disabled title={t("dialog.comingSoon")}>
        <Mail aria-hidden="true" />
        {t("filters.syncGmail")}
      </Button>
    </div>
  );
}
