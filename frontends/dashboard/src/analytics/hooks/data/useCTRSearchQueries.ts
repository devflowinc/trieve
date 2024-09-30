import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, on, useContext } from "solid-js";
import { usePagination } from "../usePagination";
import { AnalyticsParams } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { getCTRSearchQueries } from "../../api/ctr";

export const useCTRSearchQueries = ({
  params,
}: {
  params: AnalyticsParams;
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

  const queryFn = (page: number) =>
    getCTRSearchQueries(params.filter, dataset.datasetId(), page);

  createEffect(() => {
    // Preload the next page
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "ctr-search-queries",
        {
          params: params,
          page: curPage + 1,
        },
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

  const searchCTRQueriesQuery = createQuery(() => ({
    queryKey: [
      "ctr-search-queries",
      {
        params: params,
        page: pages.page(),
      },
    ],
    queryFn: () => {
      return queryFn(pages.page());
    },
  }));

  return {
    pages,
    searchCTRQueriesQuery,
    queryFn,
  };
};
