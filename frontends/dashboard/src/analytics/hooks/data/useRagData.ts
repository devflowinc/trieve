import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, useContext } from "solid-js";
import { SortOrder } from "trieve-ts-sdk";
import { usePagination } from "../usePagination";
import { getRAGQueries, getRAGUsage } from "../../api/analytics";
import { RAGAnalyticsFilter, AnalyticsParams } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

export interface RagQueriesProps {
  filter: RAGAnalyticsFilter;
  granularity: AnalyticsParams["granularity"];
}

export const useRagData = (props: RagQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const queryClient = useQueryClient();
  const [sortOrder, setSortOrder] = createSignal<SortOrder>("asc");
  const pages = usePagination();

  createEffect(() => {
    const datasetId = dataset.datasetId();
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "rag-queries",
        {
          page: curPage + 1,
          filter: props.filter,
          sortOrder: sortOrder(),
        },
      ],
      queryFn: async () => {
        const results = await getRAGQueries({
          datasetId,
          page: curPage + 1,
          filter: props.filter,
          sort_order: sortOrder(),
        });
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const ragQueriesQuery = createQuery(() => ({
    queryKey: [
      "rag-queries",
      {
        page: pages.page(),
        sortOrder: sortOrder(),
        filter: props.filter,
      },
    ],
    queryFn: () => {
      return getRAGQueries({
        datasetId: dataset.datasetId(),
        page: pages.page(),
        sort_order: sortOrder(),
        filter: props.filter,
      });
    },
  }));
  const usage = createQuery(() => ({
    queryKey: ["rag-usage", { filter: props }],
    queryFn: () => {
      return getRAGUsage(dataset.datasetId(), props.filter);
    },
  }));

  return {
    ragQueriesQuery,
    pages,
    sortOrder,
    setSortOrder,
    usage,
  };
};
