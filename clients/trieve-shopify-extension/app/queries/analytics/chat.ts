import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK, Granularity, TotalUniqueUsersResponse, RAGAnalyticsFilter, TopicsOverTimeResponse } from "trieve-ts-sdk";

export const topicsUsageQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["topicsUsage", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "topics_over_time",
        granularity: granularity,
      });

      return result as TopicsOverTimeResponse;
    },
  } satisfies QueryOptions;
};
