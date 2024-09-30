import { HeadQuery } from "shared/types";
import { createMemo, Show } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { MagicSuspense } from "../../../components/MagicBox";
import {
  HeadQueriesProps,
  useHeadQueries,
} from "../../hooks/data/useHeadQueries";

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
  const { headQueriesQuery, pages, queryFn } = useHeadQueries(props);
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
        <TanStackTable
          small
          pages={pages}
          perPage={10}
          table={tableMemo()}
          exportFn={queryFn}
        />
      </Show>
    </MagicSuspense>
  );
};
