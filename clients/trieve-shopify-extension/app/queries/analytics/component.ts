import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK, ComponentAnalyticsFilter, Granularity, TotalUniqueUsersResponse, TopPagesResponse } from "trieve-ts-sdk";

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
