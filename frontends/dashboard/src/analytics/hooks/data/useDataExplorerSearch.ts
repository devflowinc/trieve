import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQueries } from "../../api/tables";
import { createEffect, createSignal, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { subDays } from "date-fns";
import { usePagination } from "../usePagination";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { AnalyticsParams } from "shared/types";

export type sortByCols = "created_at" | "latency" | "top_score";

export const useDataExplorerSearch = () => {
  const queryClient = useQueryClient();
  const [filters, setFilters] = createStore<AnalyticsParams>({
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

  const [sortBy, setSortBy] = createSignal<sortByCols>("created_at");

  const pages = usePagination();

  const dataset = useContext(DatasetContext);

  // Get query data for next page
  createEffect(() => {
    void queryClient.prefetchQuery({
      queryKey: [
        "search-query-table",
        {
          filter: filters.filter,
          page: pages.page() + 1,
          sortBy: sortBy(),
          sortOrder: sortOrder(),
          datasetId: dataset().dataset.id,
        },
      ],
      queryFn: async () => {
        const results = await getSearchQueries(
          {
            filter: filters.filter,
            page: pages.page() + 1,
            sortBy: sortBy(),
            sortOrder: sortOrder(),
          },
          dataset().dataset.id,
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(pages.page());
        }
        return results;
      },
    });
  });

  const searchTableQuery = createQuery(() => ({
    queryKey: [
      "search-query-table",
      {
        filter: filters.filter,
        page: pages.page(),
        sortBy: sortBy(),
        sortOrder: sortOrder(),
        datasetId: dataset().dataset.id,
      },
    ],

    queryFn: () => {
      return getSearchQueries(
        {
          filter: filters.filter,
          page: pages.page(),
          sortBy: sortBy(),
          sortOrder: sortOrder(),
        },
        dataset().dataset.id,
      );
    },
  }));

  return {
    pages,
    searchTableQuery,
    sortBy,
    setSortBy,
    filters,
    setFilters,
    sortOrder,
    setSortOrder,
  };
};
