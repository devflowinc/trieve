import { ActionFunctionArgs, LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { Box } from "@shopify/polaris";
import { useSuspenseQuery } from "@tanstack/react-query";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  DatasetSettings as DatasetSettings,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { useTrieve } from "app/context/trieveContext";
import { AdminApiCaller } from "app/loaders";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { authenticate } from "app/shopify.server";
import { type Dataset } from "trieve-ts-sdk";
import { AppInstallData } from "./app.setup";
import { ResetSettings } from "app/components/ResetSettings";
import { createWebPixel, isWebPixelInstalled } from "app/queries/webPixel";
import { JudgeMeSetup } from "app/components/judgeme/JudgeMeSetup";
import { getAppMetafields, setAppMetafields } from "app/queries/metafield";


export const loader = async ({
  request,
}: LoaderFunctionArgs): Promise<{
  crawlSettings: ExtendedCrawlOptions | undefined;
  webPixelInstalled: boolean;
  devMode: boolean;
  pdpPrompt: string;
}> => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );
  setAppMetafields(fetcher, [
    {
      key: "dataset_id",
      value: key.currentDatasetId || "",
      type:"single_line_text_field"
    },
    {
      key: "api_key",
      value: key.key,
      type:"single_line_text_field"
    },
  ]).catch(console.error);

  const crawlSettings: {
    crawlSettings: ExtendedCrawlOptions | undefined;
  } = (await prisma.crawlSettings.findFirst({
    where: {
      datasetId: trieve.datasetId,
      shop: session.shop,
    },
  })) as any;

  const webPixelInstalled = await isWebPixelInstalled(fetcher, key);

  const devMode = await getAppMetafields<boolean>(fetcher, "dev_mode") || false;
  const pdpPrompt = await getAppMetafields<string>(fetcher, "pdp_prompt") || "";
  return {
    crawlSettings: crawlSettings?.crawlSettings,
    webPixelInstalled,
    devMode,
    pdpPrompt,
  };
}

export const action = async ({ request }: ActionFunctionArgs) => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );
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
    setAppMetafields(fetcher, [
      {
        key: "dataset_id",
        value: key.currentDatasetId || "",
        type:"single_line_text_field"
      },
      {
        key: "api_key",
        value: key.key,
        type:"single_line_text_field"
      },
    ]).catch(console.error);

    return { success: true };
  } else if (type === "dataset") {
    const datasetSettingsString = formData.get("dataset_settings");
    const datasetId = formData.get("dataset_id");
    const datasetSettings = JSON.parse(datasetSettingsString as string);
    await trieve.updateDataset({
      dataset_id: datasetId as string,
      server_configuration: datasetSettings,
    });
    const pdpPrompt = formData.get("pdp_prompt");
    if (pdpPrompt) {
      await setAppMetafields(fetcher, [
        {
          key: "pdp_prompt",
          value: pdpPrompt as string,
          type: "single_line_text_field",
        },
      ]);
    }
    return { success: true };
  } else if (type === "revenue_tracking") {
    await createWebPixel(fetcher, key);
    return { success: true };
  }
  return { success: false };
};

export default function Dataset() {
  const { trieve } = useTrieve();
  const { data: shopDataset } = useSuspenseQuery(shopDatasetQuery(trieve));
  const { crawlSettings, webPixelInstalled, devMode, pdpPrompt } = useLoaderData<typeof loader>();

  return (
    <Box paddingBlockStart="400">
      <DatasetSettings
        initalCrawlOptions={crawlSettings as ExtendedCrawlOptions}
        shopifyDatasetSettings={{
          devMode,
          webPixelInstalled,
          pdpPrompt,
        }}
        shopDataset={shopDataset as Dataset}
      />
      <div className="h-4"></div>
      <JudgeMeSetup />
      <div className="h-4"></div>
      <ResetSettings />
    </Box>
  );
}
