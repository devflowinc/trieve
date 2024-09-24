import { createMemo, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";
import { createQuery } from "@tanstack/solid-query";
import { getSearchCTRSummary } from "../api/ctr";

export const useCTRNeedsSetup = () => {
  const dataset = useContext(DatasetContext);
  const searchSummaryQuery = createQuery(() => ({
    queryKey: ["search-ctr-summary-info", { dataset: dataset().dataset.id }],
    queryFn: async () => {
      return getSearchCTRSummary(dataset().dataset.id);
    },
  }));
  const ctrNeedsSetup = createMemo(() => {
    dataset().dataset.id;
    if (searchSummaryQuery.isSuccess) {
      if (
        !searchSummaryQuery.data.avg_position_of_click &&
        !searchSummaryQuery.data.percent_searches_with_clicks &&
        !searchSummaryQuery.data.searches_with_clicks
      ) {
        return true;
      }
    }
    return false;
  });
  return ctrNeedsSetup;
};
