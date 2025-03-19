import { QueryOptions } from "@tanstack/react-query";
import {
  TrieveSDK,
  ComponentAnalyticsFilter,
  Granularity,
  TotalUniqueUsersResponse,
  TopPagesResponse,
  TopComponentsResponse,
  ComponentNamesResponse,
  ComponentInteractionTimeResponse,
  EventTypeAndCountsResponse,
  EventNameAndCountsResponse,
} from "trieve-ts-sdk";

export const totalUniqueUsersQuery = (
  trieve: TrieveSDK,
  filters: ComponentAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["totalUniqueUsers", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        filter: filters,
        type: "total_unique_users",
        granularity: granularity,
      });

      return result as TotalUniqueUsersResponse;
    },
  } satisfies QueryOptions;
};

export const topPagesQuery = (
  trieve: TrieveSDK,
  filters: ComponentAnalyticsFilter,
  page: number,
) => {
  return {
    queryKey: ["topPages", filters, page],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        filter: filters,
        type: "top_pages",
        page: page,
      });

      return result as TopPagesResponse;
    },
  } satisfies QueryOptions;
};

export const topComponentsQuery = (
  trieve: TrieveSDK,
  filters: ComponentAnalyticsFilter,
  page: number,
) => {
  return {
    queryKey: ["topComponents", filters, page],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        filter: filters,
        type: "top_components",
        page: page,
      });

      return result as TopComponentsResponse;
    },
  } satisfies QueryOptions;
};

export const componentNamesQuery = (trieve: TrieveSDK) => {
  return {
    queryKey: ["componentNames", trieve.datasetId],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        type: "component_names",
      });

      return result as ComponentNamesResponse;
    },
  } satisfies QueryOptions;
};

export const componentInteractionTimeQuery = (
  trieve: TrieveSDK,
  filters: ComponentAnalyticsFilter,
  granularity: Granularity,
) => {
  return {
    queryKey: ["componentInteractionTime", filters, granularity],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        filter: filters,
        type: "component_interaction_time",
        granularity: granularity,
      });

      return result as ComponentInteractionTimeResponse;
    },
  } satisfies QueryOptions;
};

export const eventNamesAndCountsQuery = (
  trieve: TrieveSDK,
  filters: ComponentAnalyticsFilter,
) => {
  return {
    queryKey: ["eventTypesAndCounts", filters],
    queryFn: async () => {
      const result = await trieve.getComponentAnalytics({
        filter: filters,
        type: "event_counts",
      });

      return result as EventNameAndCountsResponse;
    },
  } satisfies QueryOptions;
};
