import { QueryOptions } from "@tanstack/react-query";
import {
  Granularity,
  HeadQueryResponse,
  SearchAnalyticsFilter,
  SearchQueryResponse,
  SearchSortBy,
  SortOrder,
  SearchUsageGraphResponse,
  TrieveSDK,
} from "trieve-ts-sdk";
import { subDays } from "date-fns";
import { formatDateForApi } from "../../utils/formatting";

export const defaultSearchAnalyticsFilter: SearchAnalyticsFilter = {
  date_range: {
    gte: formatDateForApi(subDays(new Date(), 7)),
  },
};

export const searchUsageQuery = (
  trieve: TrieveSDK,
  filters: SearchAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["searchUsage", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getSearchAnalytics({
        filter: filters,
        type: "search_usage_graph",
        granularity: granularity,
      });
      return result as SearchUsageGraphResponse;
    },
  } satisfies QueryOptions;
};

export const headQueriesQuery = (
  trieve: TrieveSDK,
  filters: SearchAnalyticsFilter,
  page: number,
) => {
  return {
    queryKey: ["head_queries", filters, page],
    queryFn: async () => {
      const result = await trieve.getSearchAnalytics({
        filter: filters,
        type: "head_queries",
        page: page,
      });
      return result as HeadQueryResponse;
    },
  } satisfies QueryOptions;
};

export const noResultQueriesQuery = (
  trieve: TrieveSDK,
  filters: SearchAnalyticsFilter,
  page: number,
) => {
  return {
    queryKey: ["no_result_queries", filters, page],
    queryFn: async () => {
      const result = await trieve.getSearchAnalytics({
        filter: filters,
        type: "no_result_queries",
        page: page,
      });
      return result as HeadQueryResponse;
    },
  } satisfies QueryOptions;
};

export const allSearchesQuery = (
  trieve: TrieveSDK,
  filters: SearchAnalyticsFilter,
  page: number,
  has_clicks?: boolean,
  sort_by?: SearchSortBy,
  sort_order?: SortOrder,
) => {
  return {
    queryKey: ["all_searches", filters, page, has_clicks, sort_by, sort_order],
    queryFn: async () => {
      const result = await trieve.getSearchAnalytics({
        filter: filters,
        type: "search_queries",
        page: page,
        has_clicks,
        sort_by,
        sort_order,
      });
      return result as SearchQueryResponse;
    },
  } satisfies QueryOptions;
};
