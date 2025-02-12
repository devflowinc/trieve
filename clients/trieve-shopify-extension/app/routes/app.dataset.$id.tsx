import { LoaderFunctionArgs } from "@remix-run/node";
import { Link, useLoaderData } from "@remix-run/react";
import { Page, Text, Link as PolLink, Box } from "@shopify/polaris";
import { validateTrieveAuth } from "app/auth";
import {
  DatasetCrawlSettings,
  defaultCrawlOptions,
} from "app/components/CrawlSettings";
import { CrawlOptions } from "app/types";

export const loader = async (args: LoaderFunctionArgs) => {
  if (!args.params.id) {
    throw new Response("No dataset id provided", { status: 400 });
  }
  const trieveKey = await validateTrieveAuth(args);

  const datasetResponse = await fetch(
    `https://api.trieve.ai/api/dataset/${args.params.id}`,
    {
      headers: {
        Authorization: `Bearer ${trieveKey.key}`,
        "TR-Dataset": args.params.id,
      },
    },
  );

  const scrapingOptionsResponse = await fetch(
    `https://api.trieve.ai/api/crawl`,
    {
      headers: {
        Authorization: `Bearer ${trieveKey.key}`,
        "TR-Dataset": args.params.id,
      },
    },
  );

  if (!scrapingOptionsResponse.ok || !datasetResponse.ok) {
    throw new Response("Error getting information from dataset", {
      status: 404,
    });
  }

  return {
    dataset: await datasetResponse.json(),
    crawlOptions: (await scrapingOptionsResponse.json()) as {
      crawl_options: CrawlOptions | null;
    },
    trieveKey,
  };
};

export default function Dataset() {
  const { dataset, crawlOptions, trieveKey } = useLoaderData<typeof loader>();

  return (
    <Page>
      <Link to={`/app`}>
        <Box paddingBlockEnd="200">
          <PolLink>Back To Datasets</PolLink>
        </Box>
      </Link>
      <Text variant="headingXl" as="h2">
        {dataset.name}
      </Text>
      <Box paddingBlockStart="400">
        <DatasetCrawlSettings
          datasetId={dataset.id}
          trieveKey={trieveKey}
          hadCrawlEnabled={!!crawlOptions.crawl_options?.site_url}
          initalCrawlOptions={crawlOptions.crawl_options || defaultCrawlOptions}
        />
      </Box>
    </Page>
  );
}
