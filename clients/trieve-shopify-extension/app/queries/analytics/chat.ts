import { QueryOptions } from "@tanstack/react-query";
import {
  TrieveSDK,
  Granularity,
  RAGAnalyticsFilter,
  TopicsOverTimeResponse,
  CTRMetricsOverTimeResponse,
  MessagesPerUserResponse,
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
