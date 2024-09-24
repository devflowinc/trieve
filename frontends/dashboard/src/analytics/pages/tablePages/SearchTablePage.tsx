/* eslint-disable solid/reactivity */
import { format } from "date-fns";
import { SearchQueryEvent } from "shared/types";
import { FilterBar } from "../../components/FilterBar";
import { createEffect, createSignal, Show } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { parseCustomDateString } from "../../utils/formatDate";
import { useBetterNav } from "../../utils/useBetterNav";
import {
  sortByCols,
  useDataExplorerSearch,
} from "../../hooks/data/useDataExplorerSearch";
import {
  createSolidTable,
  getCoreRowModel,
  SortingState,
  Table,
} from "@tanstack/solid-table";
import { Card } from "../../components/charts/Card";
import { formatSearchMethod } from "../../utils/searchType";

const columns: SortableColumnDef<SearchQueryEvent>[] = [
  {
    accessorKey: "query",
    header: "Query",
  },
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
    accessorKey: "request_params.search_type",
    id: "search_type",
    header: "Search Method",
    cell(props) {
      return typeof props.getValue<unknown>() === "string"
        ? formatSearchMethod(props.getValue<string>())
        : "All";
    },
  },
  {
    accessorKey: "latency",
    header: "Latency",

    sortable: true,
    cell(props) {
      return props.getValue<string>() + "ms";
    },
  },
  {
    accessorKey: "top_score",
    header: "Top Score",
    sortable: true,
  },
];

export const SearchTablePage = () => {
  const navigate = useBetterNav();

  const {
    pages,
    searchTableQuery,
    sortBy,
    setSortBy,
    filters,
    setFilters,
    sortOrder,
    setSortOrder,
  } = useDataExplorerSearch();
  const [sorting, setSorting] = createSignal<SortingState>([
    {
      id: sortBy(),
      desc: sortOrder() === "desc",
    },
  ]);

  createEffect(() => {
    setSortBy(sorting()[0].id as sortByCols);
    setSortOrder(sorting()[0].desc ? "desc" : "asc");
  });

  const table = createSolidTable({
    get data() {
      return searchTableQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
      get sorting() {
        return sorting();
      },
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    manualSorting: true,
    enableSortingRemoval: false,
    onSortingChange: setSorting,
  });

  return (
    <div>
      <div class="my-4 rounded-md bg-white">
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={searchTableQuery.data}
        >
          <Card>
            <FilterBar noPadding filters={filters} setFilters={setFilters} />
            <TanStackTable
              pages={pages}
              perPage={10}
              table={table as unknown as Table<SearchQueryEvent>}
              onRowClick={(row: SearchQueryEvent) =>
                navigate(`/query/${row.id}`)
              }
            />
          </Card>
        </Show>
      </div>
    </div>
  );
};
