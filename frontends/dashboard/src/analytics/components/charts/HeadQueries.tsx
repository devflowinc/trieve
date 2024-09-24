import { AnalyticsFilter, HeadQuery } from "shared/types";
import { createMemo, Show } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useHeadQueries } from "../../hooks/data/useHeadQueries";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { MagicSuspense } from "../../../components/MagicBox";

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
  const headQueriesData = createMemo(() => {
    return useHeadQueries({
      params: props.params,
    });
  });

  const tableMemo = createMemo(() => {
    const { headQueriesQuery, pages } = headQueriesData();
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
        when={
          headQueriesData()?.headQueriesQuery.data &&
          headQueriesData().headQueriesQuery.data?.length
        }
      >
        <TanStackTable
          small
          pages={headQueriesData().pages}
          perPage={10}
          table={tableMemo()}
        />
      </Show>
    </MagicSuspense>
  );
};
