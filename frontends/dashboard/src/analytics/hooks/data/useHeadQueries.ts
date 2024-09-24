import {
  createQuery,
  CreateQueryResult,
  useQueryClient,
} from "@tanstack/solid-query";
import { createEffect, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { usePagination } from "../usePagination";
import { AnalyticsFilter, HeadQuery } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

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
    const datasetId = dataset.datasetId;
    if (prevDatasetId !== undefined && prevDatasetId !== datasetId) {
      void queryClient.invalidateQueries();
    }

    return datasetId;
  }, dataset.datasetId);

  createEffect(() => {
    // Preload the next page
    const datasetId = dataset.datasetId;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "head-queries",
        { filter: params.filter, page: curPage + 1, dataset: datasetId },
      ],
      queryFn: async () => {
        const results = await getHeadQueries(
          params.filter,
          datasetId(),
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
      return getHeadQueries(params.filter, dataset.datasetId(), pages.page());
    },
  }));

  return {
    headQueriesQuery,
    pages,
  };
};
