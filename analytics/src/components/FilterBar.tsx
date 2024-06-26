import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { Select } from "./shared/Select";
import { SetStoreFunction } from "solid-js/store";
import { For } from "solid-js";

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

export const FilterBar = (props: FilterBarProps) => {
  return (
    <div class="flex justify-between border-b border-neutral-300 bg-neutral-100 px-3 py-2">
      <div class="flex gap-2">
        <select
          value={props.filters.filter.search_method}
          onChange={(e) =>
            props.setFilters("filter", {
              ...props.filters.filter,
              search_method: e.currentTarget
                .value as AnalyticsFilter["search_method"],
            })
          }
        >
          <For each={ALL_SEARCH_METHODS}>
            {(method) => <option value={method}>{method}</option>}
          </For>
        </select>

        <select
          value={props.filters.filter.search_type}
          onChange={(e) =>
            props.setFilters("filter", {
              ...props.filters.filter,
              search_type: e.currentTarget
                .value as AnalyticsFilter["search_type"],
            })
          }
        >
          <For each={ALL_SEARCH_TYPES}>
            {(type) => <option value={type}>{type}</option>}
          </For>
        </select>
      </div>
    </div>
  );
};
