import { QueryOptions } from "@tanstack/react-query";
import {
  TrieveSDK,
  ComponentAnalyticsFilter,
  Granularity,
  TotalUniqueUsersResponse,
  TopPagesResponse,
<<<<<<< HEAD
  TopComponentsResponse,
=======
>>>>>>> bb0a48cda (feat: get component name in analytics filter)
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
      const result = await trieve.trieve.fetch(
        "/api/analytics/component_names",
        "get",
        {
          datasetId: trieve.datasetId!,
        },
      );

      return result as string[];
    },
  } satisfies QueryOptions;
};
