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
          <table class="w-full">
            <tbody>
              <tr>
                <td>Searches With Clicks</td>
                <td class="text-right font-semibold">
                  {data().searches_with_clicks.toString()}
                </td>
              </tr>
              <tr>
                <td>Percent Searches With Clicks</td>
                <td class="text-right font-semibold">
                  {Math.round(data().percent_searches_with_clicks).toString()}
                </td>
              </tr>
              <tr>
                <td>Average Click Position</td>
                <td class="text-right font-semibold">
                  {data().avg_position_of_click || "No Data"}
                </td>
              </tr>
            </tbody>
          </table>
        )}
      </Show>
    </ChartCard>
  );
};
