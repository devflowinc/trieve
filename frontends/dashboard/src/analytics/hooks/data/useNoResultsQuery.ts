import { getNoResultQueries } from "../../api/analytics";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, on, useContext } from "solid-js";
import { usePagination } from "../usePagination";
import { AnalyticsFilter } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

export const useNoResultsQueries = ({
  params,
}: {
  params: {
    filter: AnalyticsFilter;
  };
}) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(
    on(
      () => [params, dataset.datasetId()],
      () => {
        pages.resetMaxPageDiscovered();
      },
    ),
  );

  createEffect(() => {
    // Preload the next page
    const datasetId = dataset.datasetId();
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "no-result-queries",
        {
          params: params,
          page: curPage + 1,
        },
      ],
      queryFn: async () => {
        const results = await getNoResultQueries(
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

  const notResultQuery = createQuery(() => ({
    queryKey: [
      "no-result-queries",
      {
        params: params,
        page: pages.page(),
      },
    ],
    queryFn: () => {
      return getNoResultQueries(
        params.filter,
        dataset.datasetId(),
        pages.page(),
      );
    },
  }));

  return {
    pages,
    notResultQuery,
  };
};
