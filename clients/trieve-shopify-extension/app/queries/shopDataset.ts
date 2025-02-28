import { QueryOptions } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";

export const shopDatasetQuery = (trieve: TrieveSDK) => {
  return {
    queryKey: ["shop_dataset", trieve.datasetId],
    queryFn: async () => {
      const shopDataset = await trieve.getDatasetById(trieve.datasetId!);
      return shopDataset;
    },
  } satisfies QueryOptions;
};
