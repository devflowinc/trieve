import { QueryOptions } from "@tanstack/react-query";
import {
  TrieveSDK,
  RecommendationAnalyticsFilter,
  Granularity,
  RecommendationUsageGraphResponse,
  RecommendationsPerUserResponse,
  RecommendationsCTRRateResponse,
  RecommendationSortBy,
  RecommendationsEventResponse,
  SortOrder,
  RecommendationsConversionRateResponse,
  EventNameAndCountsResponse,
} from "trieve-ts-sdk";

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

export const recommendationsCTRRateQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["recommendationsCTRRate", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "recommendations_ctr_rate",
        granularity: granularity,
      });

      return result as RecommendationsCTRRateResponse;
    },
  } satisfies QueryOptions;
};

export const allRecommendationsQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
  page: number,
  has_clicks?: boolean,
  sort_by?: RecommendationSortBy,
  sort_order?: SortOrder,
) => {
  return {
    queryKey: [
      "all_recommendations",
      filters,
      page,
      has_clicks,
      sort_by,
      sort_order,
    ],
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "recommendation_queries",
        page: page,
        has_clicks,
        sort_by,
        sort_order,
      });
      return result as RecommendationsEventResponse;
    },
  } satisfies QueryOptions;
};

export const recommendationConversionRateQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["recommendationConversionRate", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "recommendation_conversion_rate",
        granularity: granularity,
      });
      return result as RecommendationsConversionRateResponse;
    },
  } satisfies QueryOptions;
};

export const recommendationEventFunnelQuery = (
  trieve: TrieveSDK,
  filters: RecommendationAnalyticsFilter,
) => {
  return {
    queryKey: ["recommendationEventFunnel", filters],
    queryFn: async () => {
      const result = await trieve.getRecommendationAnalytics({
        filter: filters,
        type: "event_funnel",
      });
      return result as EventNameAndCountsResponse;
    },
  } satisfies QueryOptions;
};
