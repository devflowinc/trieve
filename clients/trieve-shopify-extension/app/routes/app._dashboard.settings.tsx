import { ActionFunctionArgs, LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { Box } from "@shopify/polaris";
import { useSuspenseQuery } from "@tanstack/react-query";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import { DatasetSettings as DatasetSettings } from "app/components/DatasetSettings";
import { useTrieve } from "app/context/trieveContext";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { authenticate } from "app/shopify.server";
import { type Dataset } from "trieve-ts-sdk";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);

  const crawlSettings = await prisma.crawlSettings.findFirst({
    where: {
      datasetId: trieve.datasetId,
      shop: session.shop,
    },
  });

  return Response.json({ crawlSettings: crawlSettings?.crawlSettings });
};

export const action = async ({ request }: ActionFunctionArgs) => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const formData = await request.formData();
  const type = formData.get("type");
  if (type === "crawl") {
    const crawlOptions = formData.get("crawl_options");
    const datasetId = formData.get("dataset_id");
    const crawlSettings = JSON.parse(crawlOptions as string);
    await prisma.crawlSettings.upsert({
      where: {
        datasetId_shop: {
          datasetId: datasetId as string,
          shop: session.shop,
        },
      },
      update: {
        crawlSettings,
      },
      create: { crawlSettings },
    });

    const fetcher = buildAdminApiFetcherForServer(
      session.shop,
      session.accessToken!,
    );

    sendChunks(datasetId as string, key, fetcher, session, crawlSettings).catch(
      console.error,
    );
    return Response.json({ success: true });
  } else if (type === "dataset") {
    const datasetSettingsString = formData.get("dataset_settings");
    const datasetId = formData.get("dataset_id");
    const datasetSettings = JSON.parse(datasetSettingsString as string);
    await trieve.updateDataset({
      dataset_id: datasetId as string,
      server_configuration: datasetSettings,
    });

    return Response.json({ success: true });
  }

  return Response.json({ success: false });
};

export default function Dataset() {
  const { trieve } = useTrieve();
  const { data: shopDataset } = useSuspenseQuery(shopDatasetQuery(trieve));
  const { crawlSettings } = useLoaderData<typeof loader>();

  return (
    <Box paddingBlockStart="400">
      <DatasetSettings
        initalCrawlOptions={crawlSettings}
        shopDataset={shopDataset as Dataset}
      />
    </Box>
  );
}
