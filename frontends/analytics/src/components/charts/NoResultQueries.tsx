import { AnalyticsFilter, SearchQueryEvent } from "shared/types";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, on, Show, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";
import { FullScreenModal, Table, Td, Th, Tr } from "shared/ui";
import { SearchQueryEventModal } from "../../pages/TrendExplorer";
import { getNoResultQueries } from "../../api/analytics";
import { BiRegularExpand } from "solid-icons/bi";
import { QueryStringDisplay } from "../QueryStringDisplay";

interface NoResultQueriesProps {
  params: {
    filter: AnalyticsFilter;
  };
}

export const NoResultQueries = (props: NoResultQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(
    on(
      () => [props.params, dataset().dataset.id],
      () => {
        pages.resetMaxPageDiscovered();
      },
    ),
  );

  createEffect(() => {
    // Preload the next page
    const params = props.params;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "no-result-queries",
        {
          params: params,
          page: curPage + 1,
        },
      ],
      queryFn: async () => {
        const results = await getNoResultQueries(
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

  const notResultQuery = createQuery(() => ({
    queryKey: [
      "no-result-queries",
      {
        params: props.params,
        page: pages.page(),
      },
    ],
    queryFn: () => {
      return getNoResultQueries(
        props.params.filter,
        dataset().dataset.id,
        pages.page(),
      );
    },
  }));

  return (
    <>
      <div>
        <div class="gap-2">
          <div class="text-lg">No Result Queries</div>
          <div class="text-sm text-neutral-600">Searches with no results</div>
        </div>
        <Show when={notResultQuery.data?.length === 0}>
          <div class="py-8 text-center opacity-80">No Data.</div>
        </Show>
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={notResultQuery.data}
        >
          {(data) => (
            <Table
              headers={
                <Tr>
                  <Th>Query</Th>
                  <Th />
                </Tr>
              }
              class="mt-2 w-full py-2"
              data={data()}
            >
              {(query) => <QueryCard query={query} />}
            </Table>
          )}
        </Show>
      </div>
      <div class="flex justify-end pt-2">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </>
  );
};

interface QueryCardProps {
  query: SearchQueryEvent;
}
const QueryCard = (props: QueryCardProps) => {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <Tr
        onClick={() => {
          setOpen(true);
        }}
        class="cursor-pointer odd:bg-white even:bg-neutral-100 hover:underline hover:odd:bg-neutral-100/80 hover:even:bg-neutral-200/80"
      >
        <Td class="truncate">
          <QueryStringDisplay>{props.query.query}</QueryStringDisplay>
        </Td>
        <Td class="w-4 truncate text-right">
          <span class="hover:text-fuchsia-500">
            <BiRegularExpand />
          </span>
        </Td>
      </Tr>
      <FullScreenModal title={props.query.query} show={open} setShow={setOpen}>
        <SearchQueryEventModal searchEvent={props.query} />
      </FullScreenModal>
    </>
  );
};
