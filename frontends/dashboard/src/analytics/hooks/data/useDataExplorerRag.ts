import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getRagQueries } from "../../api/tables";
import { createEffect, createSignal, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { subDays } from "date-fns";
import { usePagination } from "../usePagination";
import { RAGAnalyticsFilter } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

export type RAGSortByCols =
  | "created_at"
  | "latency"
  | "hallucination_score"
  | "top_score";

export const useDataExplorerRag = () => {
  const queryClient = useQueryClient();
  const [filters, setFilters] = createStore<RAGAnalyticsFilter>({
    date_range: {
      gt: subDays(new Date(), 7),
    },
    rag_type: undefined,
  });

  const [sortOrder, setSortOrder] = createSignal<"desc" | "asc">("desc");

  const [sortBy, setSortBy] = createSignal<RAGSortByCols>("created_at");

  const pages = usePagination();

  const dataset = useContext(DatasetContext);

  // Get query data for next page
  createEffect(() => {
    void queryClient.prefetchQuery({
      queryKey: [
        "rag-query-table",
        {
          filter: filters,
          page: pages.page() + 1,
          sortBy: sortBy(),
          sortOrder: sortOrder(),
          datasetId: dataset.datasetId(),
        },
      ],
      queryFn: async () => {
        const results = await getRagQueries(
          {
            filter: filters,
            page: pages.page() + 1,
            sortBy: sortBy(),
            sortOrder: sortOrder(),
          },
          dataset.datasetId(),
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(pages.page());
        }
        return results;
      },
    });
  });

  const ragTableQuery = createQuery(() => ({
    queryKey: [
      "rag-query-table",
      {
        filter: filters,
        page: pages.page(),
        sortBy: sortBy(),
        sortOrder: sortOrder(),
        datasetId: dataset.datasetId(),
      },
    ],

    queryFn: () => {
      return getRagQueries(
        {
          filter: filters,
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
    ragTableQuery,
    sortBy,
    setSortBy,
    filters,
    setFilters,
    sortOrder,
    setSortOrder,
  };
};
