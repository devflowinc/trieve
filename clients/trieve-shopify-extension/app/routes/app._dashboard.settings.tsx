import { LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { Page, Link as PolLink, Box } from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  defaultCrawlOptions,
  DatasetSettings as DatasetSettings,
} from "app/components/DatasetSettings";
import { type Dataset } from "trieve-ts-sdk";

export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args.request);
  const trieve = sdkFromKey(key);

  const scrapingOptions = await trieve.trieve.fetch("/api/crawl", "get", {
    datasetId: key.currentDatasetId,
  });

  let shopDataset = await trieve.getDatasetById(key.currentDatasetId);

  return {
    request: args.request,
    shopDataset,
    crawlOptions: scrapingOptions[0],
  };
};

export default function Dataset() {
  const { crawlOptions, shopDataset } = useLoaderData<typeof loader>();

  const mappedCrawlOptions = crawlOptions?.crawl_options
    ? {
        ...crawlOptions.crawl_options,
        scrape_options: {
          ...crawlOptions.crawl_options.scrape_options,
          type: "shopify" as const,
        },
      }
    : defaultCrawlOptions;

  return (
    <Page>
      <Box paddingBlockStart="400">
        <DatasetSettings
          initalCrawlOptions={mappedCrawlOptions}
          shopDataset={shopDataset as Dataset}
        />
      </Box>
    </Page>
  );
}
