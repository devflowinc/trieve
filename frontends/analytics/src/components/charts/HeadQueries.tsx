import { AnalyticsFilter, HeadQuery } from "shared/types";
import { createMemo, Show } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useHeadQueries } from "../../hooks/data/useHeadQueries";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";

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
    <>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesData().headQueriesQuery.data}
      >
        <TanStackTable
          small
          pages={headQueriesData().pages}
          perPage={10}
          table={tableMemo()}
        />
      </Show>
    </>
  );
};
