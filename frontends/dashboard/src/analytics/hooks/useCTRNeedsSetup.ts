import { createMemo, useContext } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { getSearchCTRSummary } from "../api/ctr";
import { DatasetContext } from "../../contexts/DatasetContext";

export const useCTRNeedsSetup = () => {
  const dataset = useContext(DatasetContext);
  const searchSummaryQuery = createQuery(() => ({
    queryKey: ["search-ctr-summary-info", { dataset: dataset.datasetId() }],
    queryFn: async () => {
      return getSearchCTRSummary(dataset.datasetId());
    },
  }));
  const ctrNeedsSetup = createMemo(() => {
    dataset.datasetId();
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
