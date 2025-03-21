import { json, LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  defaultCrawlOptions,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { AdminApiCaller } from "app/loaders";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { authenticate } from "app/shopify.server";
import { TrieveKey } from "app/types";
import { TrieveSDK } from "trieve-ts-sdk";

export type AppInstallData = {
  currentAppInstallation: { id: string };
};

export const loader = async (args: LoaderFunctionArgs) => {
  const startCrawl = async (
    crawlOptions: ExtendedCrawlOptions,
    datasetId: string,
    session: { shop: string },
    trieveKey: TrieveKey,
    adminApi: AdminApiCaller,
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

    sendChunks(
      datasetId ?? "",
      trieveKey,
      adminApi,
      session,
      crawlOptions,
    ).catch(console.error);
  };

  const setAppMetafields = async (
    adminApi: AdminApiCaller,
    trieveKey: TrieveKey,
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
          metafieldsSetInput: [
            {
              namespace: "trieve",
              key: "dataset_id",
              value: trieveKey.currentDatasetId,
              type: "single_line_text_field",
              ownerId: appId.currentAppInstallation.id,
            },
            {
              namespace: "trieve",
              key: "api_key",
              value: trieveKey.key,
              type: "single_line_text_field",
              ownerId: appId.currentAppInstallation.id,
            },
          ],
        },
      },
    );
  };
  // You've been redirected here from app._dashboard.tsx because your trieve <-> shopify connection doesn't have a database
  const { admin, session, sessionToken } = await authenticate.admin(
    args.request,
  );

  let key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });
  if (!key) {
    throw json({ message: "No Key" }, 401);
  }

  if (!key.organizationId) {
    throw new Response("Unautorized, no organization tied to user session", {
      status: 401,
    });
  }

  const trieve = new TrieveSDK({
    baseUrl: getTrieveBaseUrlEnv(),
    apiKey: key.key,
    datasetId: key.currentDatasetId ? key.currentDatasetId : undefined,
    organizationId: key.organizationId,
    omitCredentials: true,
  });

  let datasetId = trieve.datasetId;

  let shopDataset = await trieve
    .getDatasetByTrackingId(session.shop)
    .catch(() => {
      return null;
    });

  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );

  if ((!datasetId || !shopDataset) && trieve.organizationId) {
    if (!shopDataset) {
      shopDataset = await trieve.createDataset({
        dataset_name: session.shop,
        tracking_id: session.shop,
      });

      key = await prisma.apiKey.update({
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

    if (key.currentDatasetId && key.key && session) {
      setAppMetafields(fetcher, key);
    }
  }
  datasetId = shopDataset?.id;
  if (!datasetId) {
    throw new Response("Error choosing default dataset, need to create one", {
      status: 500,
    });
  }

  startCrawl(defaultCrawlOptions, datasetId, session, key, fetcher);

  trieve.datasetId = datasetId;

  return redirect("/app");
};
