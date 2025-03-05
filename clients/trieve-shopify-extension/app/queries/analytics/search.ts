import { QueryOptions } from "@tanstack/react-query";
import {
  Granularity,
  SearchAnalyticsFilter,
  SearchUsageGraphResponse,
  TrieveSDK,
} from "trieve-ts-sdk";
import { subDays } from "date-fns";
import { formatDateForApi } from "./formatting";

export const defaultSearchAnalyticsFilter: SearchAnalyticsFilter = {
  date_range: {
    gte: formatDateForApi(subDays(new Date(), 30)),
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
