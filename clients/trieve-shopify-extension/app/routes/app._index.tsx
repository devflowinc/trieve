import { LoaderFunctionArgs } from "@remix-run/node";
import { Link, useLoaderData } from "@remix-run/react";
import {
  Page,
  Text,
  Link as PolLink,
  Box,
  Button,
  InlineGrid,
  InlineStack,
} from "@shopify/polaris";
import { initTrieveSdk, validateTrieveAuth } from "app/auth";
import {
  defaultCrawlOptions,
  DatasetSettings as DatasetSettings,
  ExtendedCrawlOptions,
  DatasetConfig,
  defaultServerEnvsConfiguration,
} from "app/components/DatasetSettings";
import { sendChunks } from "app/processors/getProducts";
import { authenticate } from "app/shopify.server";
import { TrieveKey } from "app/types";
import { request } from "http";
import { CrawlRequest, type Dataset } from "trieve-ts-sdk";

const setAppMetafields = async (admin: any, trieveKey: TrieveKey) => {
  const response = await admin.graphql(`
      #graphql
      query {
        currentAppInstallation {
          id
        }
      }
      `);

  const appId = (await response.json()) as {
    data: { currentAppInstallation: { id: string } };
  };
  await admin.graphql(
    `
    #graphql
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
        metafieldsSetInput: [
          {
            namespace: "trieve",
            key: "dataset_id",
            value: trieveKey.currentDatasetId,
            type: "single_line_text_field",
            ownerId: appId,
          },
          {
            namespace: "trieve",
            key: "api_key",
            value: trieveKey.key,
            type: "single_line_text_field",
            ownerId: appId,
          },
        ],
      },
    }
  );
};

export const loader = async (args: LoaderFunctionArgs) => {
  const { admin, session, sessionToken } = await authenticate.admin(
    args.request
  );
  const trieve = await initTrieveSdk(args);

  if (!trieve.organizationId) {
    throw new Response("Unautorized, no organization tied to user session", {
      status: 401,
    });
  }

  let datasetId = trieve.datasetId;

  let shopDataset = await trieve.getDatasetById(datasetId ?? "").catch(() => {
    return null;
  });
  if ((!datasetId || !shopDataset) && trieve.organizationId) {
    if (!shopDataset) {
      shopDataset = await trieve.createDataset({
        dataset_name: session.shop,
        tracking_id: session.shop,
      });

      await prisma.apiKey.update({
        data: {
          currentDatasetId: shopDataset.id,
        },
        where: {
          userId_shop: {
            userId: sessionToken.sub as string,
            shop: `https://${session.shop}`,
          },
        },
      });
    }

    datasetId = shopDataset?.id;
    const trieveKey = await validateTrieveAuth(args);
    setAppMetafields(admin, trieveKey);
    startCrawl(defaultCrawlOptions, datasetId, session, trieveKey, admin);
  }

  if (!datasetId) {
    throw new Response("Error choosing default dataset, need to create one", {
      status: 500,
    });
  }

  trieve.datasetId = datasetId;
  const scrapingOptions = (await trieve.trieve.fetch("/api/crawl", "get", {
    datasetId: datasetId,
  })) as unknown as CrawlRequest[];

  const appEmbedDeepLink = `https://${session.shop}/admin/themes/current/editor?context=apps&template=index&activateAppId=${process.env.SHOPIFY_GLOBAL_COMPONENT_ID}/global_component`;

  return {
    appEmbedDeepLink,
    shopDataset,
    crawlOptions: scrapingOptions[0],
  };
};

const startCrawl = async (
  crawlOptions: ExtendedCrawlOptions,
  datasetId: string,
  session: { shop: string },
  trieveKey: TrieveKey,
  admin: any
) => {
  await prisma.crawlSettings.upsert({
    create: {
      datasetId: datasetId,
      shop: session.shop,
      crawlSettings: crawlOptions,
    },
    update: {
      crawlSettings: crawlOptions,
    },
    where: {
      datasetId_shop: {
        datasetId: datasetId,
        shop: session.shop,
      },
    },
  });

  sendChunks(datasetId ?? "", trieveKey, admin, session, crawlOptions).catch(
    console.error
  );
};

export const action = async (data: LoaderFunctionArgs) => {
  const { admin, session } = await authenticate.admin(data.request);
  const trieveKey = await validateTrieveAuth(data);
  const trieve = await initTrieveSdk(data);
  let formData = await data.request.formData();
  switch (formData.get("type")) {
    case "crawl":
      const crawlOptions: ExtendedCrawlOptions = JSON.parse(
        formData.get("crawl_options") as string
      );
      const datasetId = formData.get("dataset_id") as string;

      startCrawl(crawlOptions, datasetId, session, trieveKey, admin);
      break;
    case "dataset":
      const datasetSettings: DatasetConfig =
        (JSON.parse(
          formData.get("dataset_settings") as string
        ) as DatasetConfig) ?? defaultServerEnvsConfiguration;
      const settingsDatasetId = formData.get("dataset_id") as string;
      await trieve.updateDataset({
        dataset_id: settingsDatasetId,
        server_configuration: datasetSettings,
      });

    default:
      break;
  }
  return null;
};

export default function Dataset() {
  const { appEmbedDeepLink, shopDataset, crawlOptions } =
    useLoaderData<typeof loader>();

  return (
    <Page>
      <InlineStack align="space-between">
        <Text variant="headingXl" as="h2">
          {shopDataset?.name}
        </Text>
        <Button
          external
          onClick={() => window.open(appEmbedDeepLink)}
          size="slim"
        >
          Add to theme
        </Button>
      </InlineStack>
      <Box paddingBlockStart="400">
        <DatasetSettings
          initalCrawlOptions={
            crawlOptions?.crawl_options || defaultCrawlOptions
          }
          shopDataset={shopDataset as Dataset}
        />
      </Box>
    </Page>
  );
}
