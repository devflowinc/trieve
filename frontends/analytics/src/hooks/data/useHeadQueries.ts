import {
  createQuery,
  CreateQueryResult,
  useQueryClient,
} from "@tanstack/solid-query";
import { createEffect, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { AnalyticsFilter, HeadQuery } from "shared/types";

export interface HeadQueriesData {
  headQueriesQuery: CreateQueryResult<HeadQuery[], Error>;
  pages: ReturnType<typeof usePagination>;
}

export const useHeadQueries = ({
  params,
}: {
  params: { filter: AnalyticsFilter };
}): HeadQueriesData => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect((prevDatasetId) => {
    const datasetId = dataset().dataset.id;
    if (prevDatasetId !== undefined && prevDatasetId !== datasetId) {
      void queryClient.invalidateQueries();
    }

    return datasetId;
  }, dataset().dataset.id);

  createEffect(() => {
    // Preload the next page
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "head-queries",
        { filter: params.filter, page: curPage + 1, dataset: datasetId },
      ],
      queryFn: async () => {
        const results = await getHeadQueries(
          params.filter,
          datasetId,
          curPage + 1,
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { filters: params, page: pages.page() }],
    queryFn: () => {
      return getHeadQueries(params.filter, dataset().dataset.id, pages.page());
    },
  }));

  return {
    headQueriesQuery,
    pages,
  };
};
