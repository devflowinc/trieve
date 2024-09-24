import { AnalyticsFilter, HeadQuery } from "shared/types";
import { createEffect, createMemo, Show, useContext } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { MagicSuspense } from "../../../components/MagicBox";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { usePagination } from "../../hooks/usePagination";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getHeadQueries } from "../../api/analytics";

interface HeadQueriesProps {
  params: { filter: AnalyticsFilter };
}

const columns: SortableColumnDef<HeadQuery>[] = [
  {
    accessorKey: "query",
    header: "Query",
  },
  {
    accessorKey: "count",
    header: "Frequency",
  },
];

export const HeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(() => {
    // Preload the next page
    const datasetId = dataset.datasetId();
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "head-queries",
        { filters: props.params.filter, page: curPage + 1, dataset: datasetId },
      ],
      queryFn: async () => {
        const results = await getHeadQueries(
          props.params.filter,
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
    queryKey: [
      "head-queries",
      {
        filters: props.params.filter,
        page: pages.page(),
        dataset: dataset.datasetId(),
      },
    ],
    queryFn: () => {
      return getHeadQueries(
        props.params.filter,
        dataset.datasetId(),
        pages.page(),
      );
    },
  }));

  const tableMemo = createMemo(() => {
    const table = createSolidTable({
      get data() {
        return headQueriesQuery.data || [];
      },
      state: {
        pagination: {
          pageIndex: pages.page(),
          pageSize: 10,
        },
      },
      columns,
      getCoreRowModel: getCoreRowModel(),
      manualPagination: true,
    });

    return table;
  });

  return (
    <MagicSuspense unstyled skeletonKey="headqueries">
      <Show
        fallback={<div class="py-8 text-center">No Data.</div>}
        when={headQueriesQuery.data && headQueriesQuery.data?.length}
      >
        <TanStackTable small pages={pages} perPage={10} table={tableMemo()} />
      </Show>
    </MagicSuspense>
  );
};
