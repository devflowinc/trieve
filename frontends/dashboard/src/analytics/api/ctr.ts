import { AnalyticsFilter, CTRSearchQuery } from "shared/types";
import { transformAnalyticsFilter } from "../utils/formatDate";

const apiHost = import.meta.env.VITE_API_HOST as string;

export const getSearchCTRSummary = async (
  datasetId: string,
  filters?: AnalyticsFilter,
) => {
  const response = await fetch(`${apiHost}/analytics/events/ctr`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: filters ? transformAnalyticsFilter(filters) : undefined,
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

export const getCTRSearchQueries = async (
  filter: AnalyticsFilter,
  datasetId: string,
  page: number,
) => {
  const response = await fetch(`${apiHost}/analytics/events/ctr`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filter),
      type: "searches_with_clicks",
      page,
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch ctr search queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as {
    queries: CTRSearchQuery[];
  };

  return data.queries;
};
