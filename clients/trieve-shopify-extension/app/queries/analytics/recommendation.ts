import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK, RecommendationAnalyticsFilter, Granularity, RecommendationUsageGraphResponse, RecommendationsPerUserResponse } from "trieve-ts-sdk";

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

export const recommendationsPerUserQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["recommendationsPerUser", filters, granularity], 
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "recommendations_per_user",
        granularity: granularity,
      });

      return result as RecommendationsPerUserResponse;
    },
  } satisfies QueryOptions;
};
