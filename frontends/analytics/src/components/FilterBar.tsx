import { AnalyticsParams, RequiredAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { createSignal } from "solid-js";

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
  "week",
  "month",
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
          <Select
            label={<div class="text-sm text-neutral-600">Date Range</div>}
            class="min-w-[80px] !bg-white"
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
            label={<div class="text-sm text-neutral-600">Granularity</div>}
            display={(s) => toTitleCase(s as string)}
            selected={props.filters.granularity}
            onSelected={(e) => {
              props.setFilters("granularity", e);
            }}
            options={timeFrameOptions}
            class="!bg-white"
          />
        </div>
      </div>
    </div>
  );
};
