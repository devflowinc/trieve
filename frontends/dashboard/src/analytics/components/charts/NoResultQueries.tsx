/* eslint-disable solid/reactivity */
import { AnalyticsFilter, SearchQueryEvent } from "shared/types";
import { Show } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useNoResultsQueries } from "../../hooks/data/useNoResultsQuery";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { formatSearchMethod } from "../../utils/searchType";
import { MagicSuspense } from "../../../components/MagicBox";

interface NoResultQueriesProps {
  params: {
    filter: AnalyticsFilter;
  };
}

const columns: SortableColumnDef<SearchQueryEvent>[] = [
  {
    accessorKey: "created_at",
    header: "Searched At",
    sortable: true,
    cell(props) {
      return format(
        parseCustomDateString(props.getValue<string>()),
        "M/d/yy h:mm a",
      );
    },
  },
  {
    accessorKey: "query",
    header: "Query",
  },
  {
    accessorKey: "search_type",
    header: "Search Type",
    cell(props) {
      return typeof props.getValue<unknown>() === "string"
        ? formatSearchMethod(props.getValue<string>())
        : "All";
    },
  },
  {
    accessorKey: "latency",
    header: "Latency",
    cell(props) {
      return props.getValue<number>() + "ms";
    },
  },
  {
    accessorKey: "top_score",
    header: "Top Score",
  },
];

export const NoResultQueries = (props: NoResultQueriesProps) => {
  const { notResultQuery, pages } = useNoResultsQueries({
    params: props.params,
  });
  const table = createSolidTable({
    get data() {
      return notResultQuery.data || [];
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
    <MagicSuspense skeletonKey="noresultqueries" unstyled>
      <div>
        <Show when={notResultQuery.data?.length === 0}>
          <div class="py-8 text-center opacity-80">No Data.</div>
        </Show>
        <Show when={notResultQuery.data && notResultQuery.data.length > 0}>
          <TanStackTable table={table} pages={pages} perPage={10} />
        </Show>
      </div>
    </MagicSuspense>
  );
};
