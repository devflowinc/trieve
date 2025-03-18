import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK, RecommendationAnalyticsFilter, Granularity, RecommendationUsageGraphResponse } from "trieve-ts-sdk";

export const recommendationsUsageQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["recommendationsUsage", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "recommendation_usage_graph",
        granularity: granularity,
      });
      return result as RecommendationUsageGraphResponse;
    },
  } satisfies QueryOptions;
};
