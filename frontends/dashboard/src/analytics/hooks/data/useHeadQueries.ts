import { createEffect, useContext } from "solid-js";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { usePagination } from "../../hooks/usePagination";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getHeadQueries } from "../../api/analytics";
import { AnalyticsFilter } from "shared/types";

export interface HeadQueriesProps {
  params: { filter: AnalyticsFilter };
}

export const useHeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  const queryFn = (page: number) =>
    getHeadQueries(props.params.filter, dataset.datasetId(), page);

  createEffect(() => {
    // Preload the next page
    const datasetId = dataset.datasetId();
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "head-queries",
        { filters: props.params.filter, page: curPage + 1, dataset: datasetId },
      ],
      queryFn: async () => {
        const results = await queryFn(curPage + 1);
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: [
      "head-queries",
      {
        filters: props.params.filter,
        page: pages.page(),
        dataset: dataset.datasetId(),
      },
    ],
    queryFn: () => {
      return queryFn(pages.page());
    },
  }));
  return { headQueriesQuery, queryFn, pages };
};
