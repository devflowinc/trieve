import { AnalyticsParams } from "shared/types";
import { For, Show, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { createQuery } from "@tanstack/solid-query";
import { getQueryCounts } from "../../api/analytics";
import { ChartCard } from "./ChartCard";

interface QueryCountsProps {
  filters: AnalyticsParams;
}

export const QueryCounts = (props: QueryCountsProps) => {
  const dataset = useContext(DatasetContext);

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["queryCounts", { filters: props.filters }],
    queryFn: () => {
      return getQueryCounts(props.filters, dataset().dataset.id);
    },
  }));

  return (
    <ChartCard title="Total Searches" width={5}>
      <Show
        fallback={<div class="px-7">No Data</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <For each={data()} fallback={<div class="py-8">No data</div>}>
            {(queryCount) => (
              <div class="flex justify-between py-2">
                <div>{queryCount.search_type}</div>
                <div>{queryCount.search_count}</div>
              </div>
            )}
          </For>
        )}
      </Show>
    </ChartCard>
  );
};
