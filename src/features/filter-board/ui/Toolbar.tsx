import { Mail, Search } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { TARGETS } from '@/entities/job/model/stages';
import type { Target } from '@/shared/bindings/Target';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { Field, FieldLabel, FieldLegend, FieldSet } from '@/shared/ui/field';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { clampScore, type BoardFilters } from '../model/filter';

type Props = {
  filters: BoardFilters;
  sources: string[];
  onChange: (filters: BoardFilters) => void;
  onSearch?: () => void;
};
export function Toolbar({ filters, sources, onChange, onSearch }: Props) {
  const { t } = useTranslation();
  const summary = (count: number) => (count === 0 ? t('filters.all') : t('filters.selected', { count }));
  return (
    <FieldSet className="flex flex-1 flex-row flex-wrap items-end gap-4">
      <FieldLegend className="sr-only">{t('filters.label')}</FieldLegend>
      {onSearch && (
        <Button variant="outline" className="h-9 w-64 justify-between text-noir-muted" onClick={onSearch}>
          <span className="flex items-center gap-2">
            <Search aria-hidden="true" />
            {t('command.open')}
          </span>
          <kbd className="font-mono text-xs">{t('command.shortcut')}</kbd>
        </Button>
      )}
      <Field className="w-48">
        <FieldLabel htmlFor="filter-source">{t('filters.source')}</FieldLabel>
        <Select
          multiple
          value={filters.sources}
          onValueChange={value => onChange({ ...filters, sources: value as string[] })}
        >
          <SelectTrigger id="filter-source" className="h-9 w-full">
            <SelectValue>{(value: string[]) => summary(value.length)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {sources.map(source => (
              <SelectItem key={source} value={source}>
                {source}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>
      <Field className="w-48">
        <FieldLabel htmlFor="filter-target">{t('filters.target')}</FieldLabel>
        <Select
          multiple
          value={filters.targets}
          onValueChange={value => onChange({ ...filters, targets: value as Target[] })}
        >
          <SelectTrigger id="filter-target" className="h-9 w-full">
            <SelectValue>{(value: Target[]) => summary(value.length)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {TARGETS.map(target => (
              <SelectItem key={target} value={target}>
                {t(`target.${target}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </Field>
      <Field className="w-28">
        <FieldLabel htmlFor="filter-score">{t('filters.minScore')}</FieldLabel>
        <Input
          id="filter-score"
          type="number"
          inputMode="numeric"
          min={0}
          max={100}
          step={5}
          value={filters.minScore}
          onChange={event => onChange({ ...filters, minScore: clampScore(event.target.value) })}
          className="h-9 font-mono"
        />
      </Field>
      <Field orientation="horizontal" className="h-9 w-auto items-center">
        <Checkbox
          id="filter-unread"
          checked={filters.unreadOnly}
          onCheckedChange={checked => onChange({ ...filters, unreadOnly: checked === true })}
        />
        <FieldLabel htmlFor="filter-unread">{t('filters.unreadOnly')}</FieldLabel>
      </Field>
      <Button variant="outline" className="ml-auto h-9" disabled title={t('dialog.comingSoon')}>
        <Mail aria-hidden="true" />
        {t('filters.syncGmail')}
      </Button>
    </FieldSet>
  );
}
