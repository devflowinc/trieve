import { AnalyticsParams, HeadQuery } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, For, Show, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";

interface HeadQueriesProps {
  filters: AnalyticsParams;
}

export const HeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(() => {
    // Preload the next page
    const filters = props.filters;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: ["head-queries", { filters, page: curPage + 1 }],
      queryFn: async () => {
        const results = await getHeadQueries(filters, datasetId, curPage + 1);
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { filters: props.filters, page: pages.page() }],
    queryFn: () => {
      return getHeadQueries(props.filters, dataset().dataset.id, pages.page());
    },
  }));

  return (
    <ChartCard class="px-4" width={5}>
      <div class="text-lg">Head Queries</div>
      <div class="text-sm text-neutral-600">The most popular searches.</div>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <table class="mt-2 w-full py-2">
            <thead>
              <tr>
                <th class="text-left font-semibold">Query</th>
                <th class="text-right font-semibold">Count</th>
              </tr>
            </thead>
            <tbody>
              <For each={data()}>
                {(query) => {
                  return <QueryCard query={query} />;
                }}
              </For>
            </tbody>
          </table>
        )}
      </Show>
      <div class="flex justify-end pt-2">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </ChartCard>
  );
};

interface QueryCardProps {
  query: HeadQuery;
}
const QueryCard = (props: QueryCardProps) => {
  return (
    <tr>
      <td class="truncate">{props.query.query}</td>
      <td class="text-right">{props.query.count}</td>
    </tr>
  );
};
