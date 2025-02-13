import { data, LoaderFunctionArgs } from "@remix-run/node";
import { Link, useLoaderData } from "@remix-run/react";
import { Page, Text, Link as PolLink, Box } from "@shopify/polaris";
import { validateTrieveAuth } from "app/auth";
import {
  DatasetCrawlSettings,
  defaultCrawlOptions,
} from "app/components/CrawlSettings";
import { sendChunks } from "app/processors/getProducts";
import { authenticate } from "app/shopify.server";
import { CrawlOptions } from "app/types";

export const loader = async (args: LoaderFunctionArgs) => {
  let { admin, session } = await authenticate.admin(args.request);
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
    admin,
    session,
  };
};

export const action = async (data: LoaderFunctionArgs) => {
  const { admin, session } = await authenticate.admin(data.request);
  const trieveKey = await validateTrieveAuth(data);
  const crawlOptions: CrawlOptions =
    (JSON.parse(
      (await data.request.formData()).get("crawl_options") as string,
    ) as CrawlOptions) ?? defaultCrawlOptions;
  const datasetId = data.params.id;

  await sendChunks(datasetId ?? "", trieveKey, admin, session, crawlOptions);
  return null;
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
          initalCrawlOptions={crawlOptions.crawl_options || defaultCrawlOptions}
        />
      </Box>
    </Page>
  );
}
