import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQueries } from "../../api/tables";
import { createEffect, createSignal, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { subDays } from "date-fns";
import { usePagination } from "../usePagination";
import { AnalyticsParams } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

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
    for (let i = 1; i <= 5; i++) {
      void queryClient.prefetchQuery({
        queryKey: [
          "search-query-table",
          {
            filter: filters.filter,
            page: pages.page() + i,
            sortBy: sortBy(),
            sortOrder: sortOrder(),
            datasetId: dataset.datasetId(),
          },
        ],
        queryFn: async () => {
          const results = await getSearchQueries(
            {
              filter: filters.filter,
              page: pages.page() + i,
              sortBy: sortBy(),
              sortOrder: sortOrder(),
            },
            dataset.datasetId(),
          );
          if (results.length === 0) {
            pages.setMaxPageDiscovered(pages.page() + i - 1);
          }
          return results;
        },
      });
    }
  });

  const searchTableQuery = createQuery(() => ({
    queryKey: [
      "search-query-table",
      {
        filter: filters.filter,
        page: pages.page(),
        sortBy: sortBy(),
        sortOrder: sortOrder(),
        datasetId: dataset.datasetId(),
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
        dataset.datasetId(),
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
