import { LoaderFunctionArgs } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey, TrieveKey } from "./types";
import { TrieveSDK, CreateApiUserResponse } from "trieve-ts-sdk";
import { getTrieveBaseUrlEnv } from "./env.server";
import { buildAdminApiFetcherForServer } from "./loaders/serverLoader";
import { AdminApiCaller } from "./loaders";
import { AppInstallData } from "./routes/app.setup";

export const validateTrieveAuth = async <S extends boolean = true>(
  request: LoaderFunctionArgs["request"],
  strict: S = true as S,
): Promise<S extends true ? StrongTrieveKey : TrieveKey> => {
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
      console.error(response.error);
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

  const { admin, session } = await authenticate.admin(request);

  let key = await prisma.apiKey.findFirst({
    where: {
      shop: (session.shop as string) ?? "",
    },
  });

  if (!key) {
    console.log("No key found for current shop, creating one");
    const query = await admin.graphql(
      `
      query {
        shop {
          name
          email
        }
      }
    `).catch((e) => {
        console.error(e);
        throw e;
      });

    const queryResponse = await query?.json().catch((e) => {
      console.error(e);
      throw e;
    });

    const shop_name = queryResponse?.data?.shop?.name ?? "";
    const shop_email = queryResponse?.data?.shop?.email ?? "";

    console.log("New User, creating credentials", { shop_name, shop_email });

    const userCredentialsResponse = await fetch(`${getTrieveBaseUrlEnv()}/api/auth/create_api_only_user`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "X-Shopify-Authorization": `${process.env.SHOPIFY_SECRET_KEY}`,
      },
      body: JSON.stringify({
        user_name: shop_name,
        user_email: shop_email,
      }),
    }).catch((e) => {
      console.error(e);
      throw e;
    });

    const userCredentials = await userCredentialsResponse.json() as CreateApiUserResponse;

    key = await prisma.apiKey.upsert({
      where: {
        shop: session.shop as string,
      },
      update: {
        key: userCredentials.api_key,
      },
      create: {
        organizationId: userCredentials.organization_id,
        shop: session.shop as string,
        key: userCredentials.api_key,
        createdAt: new Date(),
      }
    }).catch((e) => {
      console.error(e);
      throw e;
    });

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
        console.error("Could not get dataset by tracking id");
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

        console.log("created dataset for shop");

        key = await prisma.apiKey.update({
          data: {
            currentDatasetId: shopDataset.id,
          },
          where: {
            shop: `${session.shop}`,
          },
        });
      } else {
        key = await prisma.apiKey.update({
          data: {
            currentDatasetId: shopDataset.id,
          },
          where: {
            shop: `${session.shop}`,
          },
        });
      }

      if (key.currentDatasetId && key.key && session) {
        setAppMetafields(fetcher, {
          id: key.id,
          key: key.key,
          organizationId: key.organizationId,
          currentDatasetId: key.currentDatasetId,
        } as TrieveKey);
      }
    }
  }

  return {
    id: key.id,
    key: key.key,
    organizationId: key.organizationId,
    currentDatasetId: key.currentDatasetId,
  } as S extends true ? StrongTrieveKey : TrieveKey;
};

export const validateTrieveAuthWehbook = async <S extends boolean = true>(
  shop: string,
  strict: S = true as S,
): Promise<S extends true ? StrongTrieveKey : TrieveKey> => {
  const key = await prisma.apiKey.findFirst({
    where: {
      shop: `${shop}`,
    },
  });

  if (!key) {
    throw new Response(
      JSON.stringify({ message: "No key matching the current shop" }),
      {
        headers: {
          "Content-Type": "application/json; charset=utf-8",
        },
        status: 401,
      },
    );
  }

  if (strict && !key.currentDatasetId) {
    throw new Response(
      JSON.stringify({ message: "No dataset selected" }),
      {
        headers: {
          "Content-Type": "application/json; charset=utf-8",
        },
        status: 401,
      },
    );
  }

  return {
    id: key.id,
    key: key.key,
    organizationId: key.organizationId,
    currentDatasetId: key.currentDatasetId,
  } as S extends true ? StrongTrieveKey : TrieveKey;
};

export const sdkFromKey = (key: TrieveKey): TrieveSDK => {
  const trieve = new TrieveSDK({
    baseUrl: getTrieveBaseUrlEnv(),
    apiKey: key.key,
    datasetId: key.currentDatasetId ? key.currentDatasetId : undefined,
    organizationId: key.organizationId,
    omitCredentials: true,
  });

  return trieve;
};
