import { AnalyticsParams, SearchQueryEvent } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  createEffect,
  createSignal,
  For,
  on,
  Show,
  useContext,
} from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";
import { FullScreenModal } from "shared/ui";
import { SearchQueryEventModal } from "../../pages/TrendExplorer";
import { getNoResultQueries } from "../../api/analytics";

interface NoResultQueriesProps {
  filters: AnalyticsParams;
}

export const NoResultQueries = (props: NoResultQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(
    on(
      () => [props.filters, dataset().dataset.id],
      () => {
        pages.resetMaxPageDiscovered();
      },
    ),
  );

  createEffect(() => {
    // Preload the next page
    const filters = props.filters;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "no-result-queries",
        {
          filters,
          page: curPage + 1,
        },
      ],
      queryFn: async () => {
        const results = await getNoResultQueries(
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
      "no-result-queries",
      {
        filters: props.filters,
        page: pages.page(),
      },
    ],
    queryFn: () => {
      return getNoResultQueries(
        props.filters,
        dataset().dataset.id,
        pages.page(),
      );
    },
  }));

  return (
    <ChartCard class="flex flex-col justify-between px-4" width={5}>
      <div>
        <div class="gap-2">
          <div class="text-lg">No Result Queries</div>
          <div class="text-sm text-neutral-600">Searches with no results</div>
        </div>
        <Show
          fallback={<div class="py-8">Loading...</div>}
          when={lowConfidenceQueriesQuery.data}
        >
          {(data) => (
            <table class="mt-2 w-full py-2">
              <thead>
                <tr>
                  <th class="text-left font-semibold">Query</th>
                </tr>
              </thead>
              <tbody>
                <For
                  fallback={<div class="pt-4 text-center">No data found</div>}
                  each={data()}
                >
                  {(query) => {
                    return <QueryCard query={query} />;
                  }}
                </For>
              </tbody>
            </table>
          )}
        </Show>
      </div>
      <div class="flex justify-end pt-2">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </ChartCard>
  );
};

interface QueryCardProps {
  query: SearchQueryEvent;
}
const QueryCard = (props: QueryCardProps) => {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <tr
        onClick={() => {
          setOpen(true);
        }}
        class="cursor-pointer"
      >
        <td class="truncate">{props.query.query}</td>
      </tr>
      <FullScreenModal title={props.query.query} show={open} setShow={setOpen}>
        <SearchQueryEventModal searchEvent={props.query} />
      </FullScreenModal>
    </>
  );
};
