import { json, LoaderFunctionArgs, redirect } from "@remix-run/node";
import { validateTrieveAuth } from "app/auth";
import {
  defaultCrawlOptions,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { getTrieveBaseUrl } from "app/env";
import { sendChunks } from "app/processors/getProducts";
import { authenticate } from "app/shopify.server";
import { TrieveKey } from "app/types";
import { TrieveSDK } from "trieve-ts-sdk";

const startCrawl = async (
  crawlOptions: ExtendedCrawlOptions,
  datasetId: string,
  session: { shop: string },
  trieveKey: TrieveKey,
  admin: any,
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
    admin,
    session,
    crawlOptions,
    process.env.TRIEVE_API_URL || "https://api.trieve.ai",
  ).catch(console.error);
};

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
            ownerId: appId.data.currentAppInstallation.id,
          },
          {
            namespace: "trieve",
            key: "api_key",
            value: trieveKey.key,
            type: "single_line_text_field",
            ownerId: appId.data.currentAppInstallation.id,
          },
        ],
      },
    },
  );
};

export const loader = async (args: LoaderFunctionArgs) => {
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
    baseUrl: getTrieveBaseUrl(),
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

    setAppMetafields(admin, key);
  }
  datasetId = shopDataset?.id;
  if (!datasetId) {
    throw new Response("Error choosing default dataset, need to create one", {
      status: 500,
    });
  }

  startCrawl(defaultCrawlOptions, datasetId, session, key, admin);

  trieve.datasetId = datasetId;

  return redirect("/app/settings");
};
