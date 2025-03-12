import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK, ComponentAnalyticsFilter, Granularity } from "trieve-ts-sdk";

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

      return result;
    },
  } satisfies QueryOptions;
};