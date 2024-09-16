import { AnalyticsFilter, HeadQuery } from "shared/types";
import { Show } from "solid-js";
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
  const { headQueriesQuery, pages } = useHeadQueries({
    params: props.params,
  });
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

  return (
    <>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        <TanStackTable small pages={pages} perPage={10} table={table} />
      </Show>
    </>
  );
};
