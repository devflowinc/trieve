import { subDays } from "date-fns";
import { AnalyticsParams } from "shared/types";
import { createStore } from "solid-js/store";
import { FilterBar } from "../../components/FilterBar";
import { createSignal, Show, useContext } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { getSearchQueries } from "../../api/tables";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { Table, Td, Tr } from "shared/ui";

export const SearchTablePage = () => {
  const [analyticsFilters, setAnalyticsFilters] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: undefined, // All methods and types
      search_type: undefined,
    },
    granularity: "day",
  });

  const [sortOrder, setSortOrder] = createSignal<"desc" | "asc">("desc");

  const [sortBy, setSortBy] = createSignal<
    "created_at" | "latency" | "top_score"
  >("created_at");

  const [page, setPage] = createSignal(1);

  const dataset = useContext(DatasetContext);

  const searchTableQuery = createQuery(() => ({
    queryKey: [
      "search-query-table",
      {
        filter: analyticsFilters.filter,
        page: page(),
        sortBy: sortBy(),
        sortOrder: sortOrder(),
        datasetId: dataset().dataset.id,
      },
    ],

    queryFn: () => {
      return getSearchQueries(
        {
          filter: analyticsFilters.filter,
          page: page(),
          sortBy: sortBy(),
          sortOrder: sortOrder(),
        },
        dataset().dataset.id,
      );
    },
  }));

  return (
    <div>
      <FilterBar
        noPadding
        filters={analyticsFilters}
        setFilters={setAnalyticsFilters}
      />
      <div class="py-4">
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={searchTableQuery.data}
        >
          {(data) => (
            <Table
              fallback={<div class="py-8 text-center">No Data</div>}
              class="border"
              data={data()}
            >
              {(row) => (
                <Tr>
                  <Td>{row.query}</Td>
                </Tr>
              )}
            </Table>
          )}
        </Show>
      </div>
    </div>
  );
};
