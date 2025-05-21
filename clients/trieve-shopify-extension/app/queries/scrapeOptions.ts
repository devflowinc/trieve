import { QueryOptions } from "@tanstack/react-query";
import { defaultCrawlOptions } from "app/components/settings/DatasetSettings";
import { TrieveSDK } from "trieve-ts-sdk";

export const scrapeOptionsQuery = (trieve: TrieveSDK) => {
  return {
    queryKey: ["scrape_options", trieve.datasetId],
    queryFn: async () => {
      const crawlOptions = await trieve.trieve.fetch("/api/crawl", "get", {
        datasetId: trieve.datasetId!,
      });
      const mappedCrawlOptions = crawlOptions[0]?.crawl_options
        ? {
            ...crawlOptions[0].crawl_options,
            scrape_options: {
              ...crawlOptions[0].crawl_options.scrape_options,
              type: "shopify" as const,
            },
          }
        : defaultCrawlOptions;
      return mappedCrawlOptions;
    },
  } satisfies QueryOptions;
};
