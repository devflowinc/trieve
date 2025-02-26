/* eslint-disable solid/reactivity */
import { format } from "date-fns";
import { SearchQueryEvent } from "shared/types";
import { FilterBar } from "../../components/FilterBar";
import { createEffect, createSignal, Show, useContext } from "solid-js";
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
import { DatasetContext } from "../../../contexts/DatasetContext";
import { getSearchQueries } from "../../api/tables";

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

export const SearchTable = () => {
  const navigate = useBetterNav();
  const datasetContext = useContext(DatasetContext);

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
      <div class="pb-1 text-lg">All Searches</div>
      <div class="mb-4 rounded-md bg-white">
        <Show when={searchTableQuery.data || searchTableQuery.isLoading}>
          <Card>
            <FilterBar noPadding filters={filters} setFilters={setFilters} />
            <div class="mt-4 overflow-x-auto">
              <TanStackTable
                class="overflow-hidden"
                pages={pages}
                perPage={10}
                table={table as unknown as Table<SearchQueryEvent>}
                onRowClick={(row: SearchQueryEvent) =>
                  navigate(
                    `/dataset/${datasetContext.datasetId()}/analytics/query/${
                      row.id
                    }`,
                  )
                }
                exportFn={(page: number) =>
                  getSearchQueries(
                    {
                      filter: filters.filter,
                      page: page,
                      sortBy: sortBy(),
                      sortOrder: sortOrder(),
                    },
                    datasetContext.datasetId(),
                  )
                }
              />
              <Show when={searchTableQuery.data?.length === 0}>
                <div class="py-8 text-center">No Data.</div>
              </Show>
            </div>
          </Card>
        </Show>
      </div>
    </div>
  );
};
