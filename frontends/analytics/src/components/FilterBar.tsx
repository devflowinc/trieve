import { AnalyticsParams, RequiredAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { DateRangePicker, Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";

const ALL_SEARCH_METHODS: RequiredAnalyticsFilter["search_method"][] = [
  "hybrid",
  "fulltext",
  "semantic",
];

const ALL_SEARCH_TYPES: RequiredAnalyticsFilter["search_type"][] = [
  "search",
  "autocomplete",
  "search_over_groups",
  "search_within_groups",
];

interface FilterBarProps {
  filters: AnalyticsParams;
  setFilters: SetStoreFunction<AnalyticsParams>;
}

export const timeFrameOptions: AnalyticsParams["granularity"][] = [
  "day",
  "hour",
  "minute",
  "second",
];

export type DateRangeOption = {
  date: Date;
  label: string;
};

export const dateRanges: DateRangeOption[] = [
  {
    label: "Past Hour",
    date: subHours(new Date(), 1),
  },
  {
    label: "Past Day",
    date: subDays(new Date(), 1),
  },
  {
    label: "Past Week",
    date: subDays(new Date(), 7),
  },
];

export const FilterBar = (props: FilterBarProps) => {
  return (
    <div class="flex justify-between border-neutral-400 px-3 py-2">
      <div class="flex items-center gap-2">
        <div>
          <Select
            label={<div class="text-sm text-neutral-600">Search Type</div>}
            class="min-w-[200px] !bg-white"
            display={(s) => toTitleCase(s)}
            selected={props.filters.filter.search_method}
            onSelected={(e) =>
              props.setFilters("filter", {
                ...props.filters.filter,
                search_method: e,
              })
            }
            options={ALL_SEARCH_METHODS}
          />
        </div>

        <div>
          <Select
            label={<div class="text-sm text-neutral-600">Search Method</div>}
            class="min-w-[180px] !bg-white"
            display={(s) => toTitleCase(s)}
            selected={props.filters.filter.search_type}
            onSelected={(e) =>
              props.setFilters("filter", {
                ...props.filters.filter,
                search_type: e,
              })
            }
            options={ALL_SEARCH_TYPES}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <div>
          <DateRangePicker
            label="Date Range"
            value={props.filters.filter.date_range}
            onChange={(e) => props.setFilters("filter", "date_range", e)}
            onGranularitySuggestion={(g) => props.setFilters("granularity", g)}
          />
        </div>
      </div>
    </div>
  );
};
