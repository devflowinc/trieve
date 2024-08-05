import {
  AnalyticsFilter,
  SearchQueryResponse,
  SearchSortBy,
  SortOrder,
} from "shared/types";
import { transformAnalyticsFilter } from "../utils/formatDate";
import { apiHost } from "../utils/apiHost";

type SearchQueriesTablesParams = {
  filter?: AnalyticsFilter;
  page?: number;
  sortBy?: SearchSortBy;
  sortOrder?: SortOrder;
};

export const getSearchQueries = async (
  params: SearchQueriesTablesParams,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: params.filter
        ? transformAnalyticsFilter(params.filter)
        : undefined,
      page: params.page,
      sort_by: params.sortBy,
      type: "search_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryResponse;
  return data.queries;
};
