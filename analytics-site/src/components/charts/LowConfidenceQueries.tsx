import { AnalyticsParams, SearchQueryEvent } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, For, Show, useContext } from "solid-js";
import { getLowConfidenceQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";

interface LowConfidenceQueriesProps {
  filters: AnalyticsParams;
}

export const LowConfidenceQueries = (props: LowConfidenceQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(() => {
    // Preload the next page
    const filters = props.filters;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: ["low-confidence-queries", { filters, page: curPage + 1 }],
      queryFn: async () => {
        const results = await getLowConfidenceQueries(
          filters,
          datasetId,
          curPage + 1,
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const lowConfidenceQueriesQuery = createQuery(() => ({
    queryKey: [
      "low-confidence-queries",
      { filters: props.filters, page: pages.page() },
    ],
    queryFn: () => {
      return getLowConfidenceQueries(
        props.filters,
        dataset().dataset.id,
        pages.page(),
      );
    },
  }));

  return (
    <ChartCard class="px-4" width={3}>
      <div class="text-lg">Low Confidence Queries</div>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={lowConfidenceQueriesQuery.data}
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
      <div class="flex justify-end">
        <PaginationButtons size={24} pages={pages} />
      </div>
    </ChartCard>
  );
};

interface QueryCardProps {
  query: SearchQueryEvent;
}
const QueryCard = (props: QueryCardProps) => {
  return (
    <div class="flex justify-between">
      <div class="truncate">{props.query.query}</div>
      <div class="truncate">{props.query.search_type}</div>
    </div>
  );
};
