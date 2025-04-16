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

const setAppMetafields = async (
  adminApi: AdminApiCaller,
  valuesToSet: {
    key: string;
    value: string;
  }[],
) => {
  const response = await adminApi<AppInstallData>(`
      #graphql
      query {
        currentAppInstallation {
          id
        }
      }
      `);

  if (response.error) {
    throw response.error;
  }

  const appId = response.data;

  await adminApi(
    `#graphql
    mutation CreateAppDataMetafield($metafieldsSetInput: [MetafieldsSetInput!]!) {
        metafieldsSet(metafields: $metafieldsSetInput) {
          metafields {
            id
            namespace
            key
          }
          userErrors {
            field
            message
          }
        }
      }
    `,
    {
      variables: {
        metafieldsSetInput: valuesToSet.map((value) => ({
          namespace: "trieve",
          key: value.key,
          value: value.value,
          type: "single_line_text_field",
          ownerId: appId.currentAppInstallation.id,
        })),
      },
    },
  );
};

type Metafields = {
  currentAppInstallation: {
    metafields: {
      nodes: {
        id: string;
        namespace: string;
        key: string;
        value: string;
      }[];
    };
  };
};

const getAppMetafields = async (adminApi: AdminApiCaller) => {
  const response = await adminApi<Metafields>(`
    #graphql
    query {
      currentAppInstallation {
        metafields(first: 10) {
          nodes {
            id
            namespace
            key
            value
          }
        }
      }
    }
    `);

  if (response.error) {
    throw response.error;
  }

  return response.data.currentAppInstallation.metafields.nodes;
};

export const loader = async ({
  request,
}: LoaderFunctionArgs): Promise<{
  crawlSettings: ExtendedCrawlOptions | undefined;
  webPixelInstalled: boolean;
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
    },
    {
      key: "api_key",
      value: key.key,
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

  return {
    crawlSettings: crawlSettings?.crawlSettings,
    webPixelInstalled,
  };
};

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
      },
      {
        key: "api_key",
        value: key.key,
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
  const { crawlSettings, webPixelInstalled } = useLoaderData<typeof loader>();

  return (
    <Box paddingBlockStart="400">
      <DatasetSettings
        initalCrawlOptions={crawlSettings as ExtendedCrawlOptions}
        webPixelInstalled={webPixelInstalled}
        shopDataset={shopDataset as Dataset}
      />
      <div className="h-4"></div>
      <JudgeMeSetup />
      <div className="h-4"></div>
      <ResetSettings />
    </Box>
  );
}
