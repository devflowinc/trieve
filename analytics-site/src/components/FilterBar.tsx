import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { createSignal } from "solid-js";

const ALL_SEARCH_METHODS: AnalyticsFilter["search_method"][] = [
  "fulltext",
  "hybrid",
  "semantic",
];

const ALL_SEARCH_TYPES: AnalyticsFilter["search_type"][] = [
  "search",
  "search_over_groups",
  "search_within_groups",
  "autocomplete",
];

interface FilterBarProps {
  filters: AnalyticsParams;
  setFilters: SetStoreFunction<AnalyticsParams>;
}

const timeFrameOptions: AnalyticsParams["granularity"][] = [
  "day",
  "week",
  "month",
  "hour",
  "minute",
  "second",
];

type DateRangeOption = {
  date: Date;
  label: string;
};

const dateRanges: DateRangeOption[] = [
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
  {
    label: "Past Month",
    date: subDays(new Date(), 30),
  },
];

export const FilterBar = (props: FilterBarProps) => {
  const [dateSelection, setDateSelection] = createSignal<DateRangeOption>(
    dateRanges[0],
  );
  return (
    <div class="flex justify-between border-b border-neutral-300 bg-neutral-100/90 px-3 py-2">
      <div class="flex items-center gap-2">
        <div>
          <Select
            label=<div class="text-xs text-neutral-600">Search Type</div>
            class="min-w-[200px]"
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
            label=<div class="text-xs text-neutral-600">Search Method</div>
            class="min-w-[180px]"
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
          <Select
            label=<div class="text-xs text-neutral-600">Date Range</div>
            class="min-w-[80px]"
            display={(s) => s.label}
            selected={dateSelection()}
            onSelected={(e) => {
              setDateSelection(e);
              props.setFilters("filter", {
                ...props.filters.filter,
                date_range: {
                  gt: e.date,
                },
              });
            }}
            options={dateRanges}
          />
        </div>
        <div>
          <Select
            label=<div class="text-xs text-neutral-600">Granularity</div>
            display={(s) => toTitleCase(s as string)}
            selected={props.filters.granularity}
            onSelected={(e) => {
              props.setFilters("granularity", e);
            }}
            options={timeFrameOptions}
          />
        </div>
      </div>
    </div>
  );
};
