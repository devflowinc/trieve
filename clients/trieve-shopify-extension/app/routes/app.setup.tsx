import { json, LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  defaultCrawlOptions,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { AdminApiCaller } from "app/loaders";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { trackUserLinked } from "app/processors/shopifyTrackers";
import { authenticate } from "app/shopify.server";
import { TrieveKey } from "app/types";
import { TrieveSDK } from "trieve-ts-sdk";

export type AppInstallData = {
  currentAppInstallation: { id: string };
};

export const loader = async (args: LoaderFunctionArgs) => {
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

  const { session, sessionToken } = await authenticate.admin(args.request);

  let key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });
  if (!key) {
    throw new Response(
      JSON.stringify({
        message: "No key matching the current user (sessionToken.sub)",
      }),
      {
        headers: {
          "Content-Type": "application/json; charset=utf-8",
        },
        status: 401,
      },
    );
  }

  if (!key.organizationId) {
    throw new Response(
      JSON.stringify({
        message:
          "No organization matching the current key (key.organizationId)",
      }),
      {
        headers: {
          "Content-Type": "application/json; charset=utf-8",
        },
        status: 401,
      },
    );
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
    } else {
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

  let crawlSettings = await prisma.crawlSettings.findFirst({
    where: {
      datasetId: key.currentDatasetId,
      shop: session.shop,
    },
  });

  if (!crawlSettings) {
    crawlSettings = await prisma.crawlSettings.create({
      data: {
        datasetId: key.currentDatasetId,
        shop: session.shop,
        crawlSettings: defaultCrawlOptions,
      },
    });
  }

  datasetId = shopDataset?.id;
  if (!datasetId) {
    throw new Response("Error choosing default dataset, need to create one", {
      status: 500,
    });
  }

  const crawlOptions = crawlSettings.crawlSettings as ExtendedCrawlOptions;

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

  trackUserLinked({
    organization_id: key.organizationId,
    store_name: session.shop,
  }, key).catch(console.error);

  sendChunks(
    datasetId ?? "",
    key,
    fetcher,
    session,
    crawlOptions,
  ).catch(console.error);

  trieve.datasetId = datasetId;
  console.log("redirecting to app!");
  return redirect("/app");
};
