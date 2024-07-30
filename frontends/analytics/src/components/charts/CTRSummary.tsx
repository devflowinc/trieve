import { AnalyticsParams } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery } from "@tanstack/solid-query";
import { getSearchCTRSummary } from "../../api/ctr";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";

interface SearchCTRStatsProps {
  params: AnalyticsParams;
}
export const CTRSummary = (props: SearchCTRStatsProps) => {
  const dataset = useContext(DatasetContext);
  const searchSummaryQuery = createQuery(() => ({
    queryKey: [
      "search-ctr-summary",
      { filters: props.params.filter, dataset: dataset().dataset.id },
    ],
    queryFn: async () => {
      return getSearchCTRSummary(props.params.filter, dataset().dataset.id);
    },
  }));

  return (
    <ChartCard title="Summary" class="min-w-[300px]">
      <Show fallback={<div>Loading...</div>} when={searchSummaryQuery.data}>
        {(data) => (
          <div>
            <div>
              Searches With Clicks: {data().searches_with_clicks.toString()}
            </div>
            <div>
              Percent Searches With Clicks:{" "}
              {Math.round(data().percent_searches_with_clicks).toString()}%
            </div>
            <div>
              Average Click Positition:{" "}
              {data().avg_position_of_click || "No Data"}
            </div>
          </div>
        )}
      </Show>
    </ChartCard>
  );
};
