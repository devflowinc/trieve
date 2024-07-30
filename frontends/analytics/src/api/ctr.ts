import { AnalyticsFilter } from "shared/types";
import { apiHost } from "../utils/apiHost";
import { transformAnalyticsFilter } from "../utils/formatDate";

export const getSearchCTRSummary = async (
  filters: AnalyticsFilter,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/ctr`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      type: "search_ctr_metrics",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch no result queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as {
    searches_with_clicks: number;
    percent_searches_with_clicks: number;
    avg_position_of_click: number | null;
  };
  return data;
};
