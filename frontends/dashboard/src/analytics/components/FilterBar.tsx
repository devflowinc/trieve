import { AnalyticsParams, RequiredAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { DateRangePicker, Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { cn } from "shared/utils";
import { getQueryRatingFilter } from "./RAGFilterBar";

const ALL_SEARCH_METHODS: (RequiredAnalyticsFilter["search_method"] | "all")[] =
  ["all", "hybrid", "fulltext", "semantic"];

const ALL_SEARCH_TYPES: (RequiredAnalyticsFilter["search_type"] | "all")[] = [
  "all",
  "search",
  "autocomplete",
  "search_over_groups",
  "search_within_groups",
];

interface FilterBarProps {
  filters: AnalyticsParams;
  setFilters: SetStoreFunction<AnalyticsParams>;
  noPadding?: boolean;
}

export const timeFrameOptions: AnalyticsParams["granularity"][] = [
  "month",
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
    <div
      class={cn(
        "flex flex-col justify-between border-neutral-400 md:flex-row",
        !props.noPadding && "px-3 py-2",
      )}
    >
      <div class="flex items-center gap-2">
        <div>
          <Select
            label={<div class="text-sm text-neutral-600">Search Method</div>}
            class="min-w-[200px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={props.filters.filter.search_method}
            onSelected={(e) =>
              props.setFilters("filter", {
                ...props.filters.filter,
                search_method: e === "all" ? undefined : e,
              })
            }
            options={ALL_SEARCH_METHODS}
          />
        </div>

        <div>
          <Select
            label={<div class="text-sm text-neutral-600">Search Type</div>}
            class="min-w-[180px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={props.filters.filter.search_type}
            onSelected={(e) =>
              props.setFilters("filter", {
                ...props.filters.filter,
                search_type: e === "all" ? undefined : e,
              })
            }
            options={ALL_SEARCH_TYPES}
          />
        </div>
        <div>
          <Select
            label={<div class="text-sm text-neutral-600">Query Rating</div>}
            class="min-w-[200px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={
              props.filters.filter.query_rating === undefined
                ? "neutral"
                : props.filters.filter.query_rating.gte
                  ? "thumbs_up"
                  : "thumbs_down"
            }
            onSelected={(e) =>
              props.setFilters("filter", {
                ...props.filters.filter,
                query_rating: getQueryRatingFilter(e),
              })
            }
            options={["thumbs_up", "thumbs_down", "neutral"]}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <div>
          <DateRangePicker
            label="Date Range"
            value={props.filters.filter.date_range}
            onChange={(e) => props.setFilters("filter", "date_range", e)}
            initialSelectedPresetId={7}
            onGranularitySuggestion={(granularity) =>
              props.setFilters("granularity", granularity)
            }
          />
        </div>
      </div>
    </div>
  );
};
