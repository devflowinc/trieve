import { AnalyticsFilter } from "shared/types";
import { getSearchCTRSummary } from "../../api/ctr";
import { useContext } from "solid-js";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { createQuery } from "@tanstack/solid-query";

export interface SearchCTRStatsProps {
  filter: AnalyticsFilter;
}

export const useCTRSummary = (props: SearchCTRStatsProps) => {
  const dataset = useContext(DatasetContext);

  const searchSummaryQuery = createQuery(() => ({
    queryKey: [
      "search-ctr-summary",
      { filters: props.filter, dataset: dataset.datasetId() },
    ],
    queryFn: async () => {
      return getSearchCTRSummary(dataset.datasetId(), props.filter);
    },
  }));

  return {
    searchSummaryQuery,
  };
};
