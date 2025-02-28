import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";

export const usageQuery = (trieve: TrieveSDK) => {
  return {
    queryKey: ["usage", trieve.datasetId],
    queryFn: async () => {
      const usage = await trieve.getDatasetUsageById(trieve.datasetId!);
      return usage;
    },
  } satisfies QueryOptions;
};
