import { AnalyticsFilter, HeadQuery } from "shared/types";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, For, Show, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";

interface HeadQueriesProps {
  params: { filter: AnalyticsFilter };
}

export const HeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect((prevDatasetId) => {
    const datasetId = dataset().dataset.id;
    if (prevDatasetId !== undefined && prevDatasetId !== datasetId) {
      void queryClient.invalidateQueries();
    }

    return datasetId;
  }, dataset().dataset.id);

  createEffect(() => {
    // Preload the next page
    const params = props.params;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: ["head-queries", { filter: params.filter, page: curPage + 1 }],
      queryFn: async () => {
        const results = await getHeadQueries(
          params.filter,
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

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { filters: props.params, page: pages.page() }],
    queryFn: () => {
      return getHeadQueries(
        props.params.filter,
        dataset().dataset.id,
        pages.page(),
      );
    },
  }));

  return (
    <>
      <Show
        when={
          headQueriesQuery.status === "success" &&
          headQueriesQuery.data.length === 0
        }
      >
        <div class="py-4 text-center">
          <div class="text-lg">No queries found</div>
          <div class="text-sm text-neutral-600">
            There are no queries to display.
          </div>
        </div>
      </Show>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <table class="mt-2 w-full py-2">
            <thead>
              <Show when={data().length > 0}>
                <tr>
                  <th class="text-left font-semibold">Query</th>
                  <th class="text-right font-semibold">Count</th>
                </tr>
              </Show>
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
    </>
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
