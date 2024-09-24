import { AnalyticsFilter } from "shared/types";
import { createQuery } from "@tanstack/solid-query";
import { getSearchCTRSummary } from "../../api/ctr";
import { Show, useContext } from "solid-js";
import { useCTRNeedsSetup } from "../../hooks/useCTRNeedsSetup";
import { DatasetContext } from "../../../contexts/DatasetContext";

interface SearchCTRStatsProps {
  filter: AnalyticsFilter;
}
export const CTRSummary = (props: SearchCTRStatsProps) => {
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

  const ctrNeedsSetup = useCTRNeedsSetup();

  return (
    <Show fallback={<div>Loading...</div>} when={searchSummaryQuery.data}>
      {(data) => (
        <Show when={!ctrNeedsSetup()}>
          <div class="h-2" />
          <table class="w-full">
            <tbody>
              <tr>
                <td>Searches With Clicks</td>
                <td class="text-right font-medium">
                  {data().searches_with_clicks.toString()}
                </td>
              </tr>
              <tr>
                <td>Percent Searches With Clicks</td>
                <td class="text-right font-medium">
                  {Math.round(data().percent_searches_with_clicks).toString()}
                </td>
              </tr>
              <tr>
                <td>Average Click Position</td>
                <td class="text-right font-medium">
                  {data().avg_position_of_click || "No Data"}
                </td>
              </tr>
            </tbody>
          </table>
        </Show>
      )}
    </Show>
  );
};
