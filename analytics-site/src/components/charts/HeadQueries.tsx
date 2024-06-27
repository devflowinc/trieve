import { AnalyticsParams, HeadQuery } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, For, Show, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";

interface HeadQueriesProps {
  filters: AnalyticsParams;
}

export const HeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const [page, setPage] = createSignal(1);
  const [maxPage, setMaxPage] = createSignal(1);
  const queryClient = useQueryClient();

  createEffect(() => {
    // Preload the next page
    const filters = props.filters;
    const datasetId = dataset().dataset.id;
    const curPage = page();
    void queryClient.prefetchQuery({
      queryKey: ["head-queries", { filters, page: curPage + 1 }],
      queryFn: async () => {
        const results = await getHeadQueries(filters, datasetId, curPage + 1);
        if (results.length === 0) {
          setMaxPage(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { filters: props.filters, page: page() }],
    queryFn: () => {
      return getHeadQueries(props.filters, dataset().dataset.id, page());
    },
  }));

  return (
    <ChartCard class="px-4" width={3}>
      <div class="text-lg">Head Queries</div>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <div class="py-2">
            <For each={data()}>
              {(query) => {
                return <QueryCard query={query} />;
              }}
            </For>
          </div>
        )}
      </Show>
      <div class="flex justify-between">
        <button onClick={() => setPage((prev) => prev - 1)}>Prev page</button>
        <button onClick={() => setPage((prev) => prev + 1)}>Next page</button>
      </div>
    </ChartCard>
  );
};

interface QueryCardProps {
  query: HeadQuery;
}
const QueryCard = (props: QueryCardProps) => {
  return (
    <div class="flex justify-between">
      <div>{props.query.query}</div>
      <div>{props.query.count}</div>
    </div>
  );
};
