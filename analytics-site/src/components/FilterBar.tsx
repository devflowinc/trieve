import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";

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

export const FilterBar = (props: FilterBarProps) => {
  return (
    <div class="flex justify-between border-b border-neutral-300 bg-neutral-100 px-3 py-2">
      <div class="flex gap-2">
        <Select
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

        <Select
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

      <Select
        display={(s) => toTitleCase(s as string)}
        selected={props.filters.granularity}
        onSelected={(e) => {
          props.setFilters("granularity", e);
        }}
        options={timeFrameOptions}
      />
    </div>
  );
};
