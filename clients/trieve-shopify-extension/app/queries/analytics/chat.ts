import { QueryOptions } from "@tanstack/react-query";
import {
  TrieveSDK,
  Granularity,
  RAGAnalyticsFilter,
  TopicsOverTimeResponse,
  CTRMetricsOverTimeResponse,
  MessagesPerUserResponse,
  RAGSortBy,
  SortOrder,
  TopicQueriesResponse,
  ChatAverageRatingResponse,
  ChatConversionRateResponse,
  ChatRevenueResponse,
  EventNameAndCountsResponse,
  TopicAnalyticsFilter,
  PopularChatsResponse,
} from "trieve-ts-sdk";

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

export const topicsCTRRateQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["topicsCTRRate", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "ctr_metrics_over_time",
        granularity: granularity,
      });

      return result as CTRMetricsOverTimeResponse;
    },
  } satisfies QueryOptions;
};

export const messagesPerUserQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["messagesPerUser", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "messages_per_user",
        granularity: granularity,
      });

      return result as MessagesPerUserResponse;
    },
  } satisfies QueryOptions;
};

export const allChatsQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  page: number,
  has_clicks?: boolean,
  sort_by?: RAGSortBy,
  sort_order?: SortOrder,
) => {
  return {
    queryKey: ["all_chats", filters, page, has_clicks, sort_by, sort_order],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "topic_queries",
        page: page,
        has_clicks,
        sort_by,
        sort_order,
      });
      return result as TopicQueriesResponse;
    },
  } satisfies QueryOptions;
};

export const chatAverageRatingQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["chatAverageRating", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "chat_average_rating",
        granularity: granularity,
      });
      return result as ChatAverageRatingResponse;
    },
  } satisfies QueryOptions;
};

export const chatConversionRateQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["chatConversionRate", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "chat_conversion_rate",
        granularity: granularity,
      });
      return result as ChatConversionRateResponse;
    },
  } satisfies QueryOptions;
};

export const chatRevenueQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
  granularity: Granularity,
  direct: boolean,
) => {
  return {
    queryKey: ["chatRevenue", filters, granularity, direct],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "chat_revenue",
        granularity: granularity,
        direct,
      });
      return result as ChatRevenueResponse;
    },
  } satisfies QueryOptions;
};

export const chatEventFunnelQuery = (
  trieve: TrieveSDK,
  filters: RAGAnalyticsFilter,
) => {
  return {
    queryKey: ["chatEventFunnel", filters],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "event_funnel",
      });
      return result as EventNameAndCountsResponse;
    },
  } satisfies QueryOptions;
};

export const popularChatsQuery = (
  trieve: TrieveSDK,
  filters: TopicAnalyticsFilter,
  page: number,
) => {
  return {
    queryKey: ["popularChats", filters, page],
    queryFn: async () => {
      const result = await trieve.getRagAnalytics({
        filter: filters,
        type: "popular_chats",
        page: page,
      });
      return result as PopularChatsResponse;
    },
  } satisfies QueryOptions;
};
