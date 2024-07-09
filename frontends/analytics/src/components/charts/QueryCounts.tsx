import { AnalyticsParams, SearchTypeCount } from "shared/types";
import { For, Show, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { createQuery } from "@tanstack/solid-query";
import { getQueryCounts } from "../../api/analytics";
import { ChartCard } from "./ChartCard";

interface QueryCountsProps {
  filters: AnalyticsParams;
}

const displaySearchType = (type: SearchTypeCount["search_type"]) => {
  switch (type) {
    case "search":
      return "Search";
    case "autocomplete":
      return "Autocomplete";
    case "search_over_groups":
      return "Search Over Groups";
    case "search_within_groups":
      return "Search Within Groups";
  }
};

export const QueryCounts = (props: QueryCountsProps) => {
  const dataset = useContext(DatasetContext);

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["queryCounts", { filters: props.filters }],
    queryFn: () => {
      return getQueryCounts(props.filters, dataset().dataset.id);
    },
  }));

  return (
    <ChartCard class="flex flex-col justify-between px-4" width={10}>
      <div>
        <div class="flex items-baseline justify-start gap-4">
          <div class="text-lg">Total Searches</div>
          <div class="text-sm text-neutral-600">
            Total Count of Queries by Type
          </div>
        </div>
        <Show
          fallback={<div class="py-8">Loading...</div>}
          when={headQueriesQuery.data}
        >
          {(data) => (
            <div class="flex justify-around gap-2 py-2">
              <For each={data()}>
                {(search) => {
                  return (
                    <div class="text-center">
                      <div>{displaySearchType(search.search_type)}</div>
                      <div class="text-lg font-semibold">
                        {search.search_count}
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          )}
        </Show>
      </div>
    </ChartCard>
  );
};
