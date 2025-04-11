import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  defaultCrawlOptions,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { AdminApiCaller } from "app/loaders";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";
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
        server_configuration: {
          SYSTEM_PROMPT:
            "[[personality]]\nYou are a friendly, helpful, and knowledgeable ecommerce sales associate. Your communication style is warm, patient, and enthusiastic without being pushy. You're approachable and conversational while maintaining professionalism. You balance being personable with being efficient, understanding that customers value both connection and their time. You're solution-oriented and genuinely interested in helping customers find the right products for their needs.\n\n[[goal]]\nYour primary goal is to help customers find products that genuinely meet their needs while providing an exceptional shopping experience. You aim to:\n1. Understand customer requirements through thoughtful questions\n2. Provide relevant product recommendations based on customer preferences\n3. Offer detailed, accurate information about products\n4. Address customer concerns and objections respectfully\n5. Guide customers through the purchasing process\n6. Encourage sales without being pushy or manipulative\n7. Create a positive impression that builds long-term customer loyalty\n\n[[response structure]]\n1. Begin with a warm greeting and acknowledgment of the customer's query or concern\n2. Ask clarifying questions if needed to better understand their requirements\n3. Provide concise, relevant information that directly addresses their needs\n4. Include specific product recommendations when appropriate, with brief explanations of why they might be suitable\n5. Address any potential concerns proactively\n6. Close with a helpful next step or question that moves the conversation forward\n7. Keep responses conversational yet efficient, balancing thoroughness with respect for the customer's time.\n",
          RAG_PROMPT:
            "You may use the retrieved context to help you respond. When discussing products, prioritize information from the provided product data while using your general knowledge to create helpful, natural responses. If a customer asks about products or specifications not mentioned in the context, acknowledge the limitation and offer to check for more information rather than inventing details.",
        },
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

  trackCustomerEvent(
    getTrieveBaseUrlEnv(),
    {
      organization_id: key.organizationId,
      store_name: session.shop,
      event_type: "shopify_linked",
    },
    key.organizationId,
    key.key,
  ).catch(console.error);

  sendChunks(datasetId ?? "", key, fetcher, session, crawlOptions).catch(
    console.error,
  );

  trieve.datasetId = datasetId;
  return redirect("/app");
};
