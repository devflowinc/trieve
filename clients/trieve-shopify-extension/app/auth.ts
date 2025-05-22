import { LoaderFunctionArgs } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey, TrieveKey } from "./types";
import { TrieveSDK, CreateApiUserResponse } from "trieve-ts-sdk";
import { getTrieveBaseUrlEnv } from "./env.server";
import { buildAdminApiFetcherForServer } from "./loaders/serverLoader";
import { AdminApiCaller } from "./loaders";
import { AppInstallData } from "./routes/app.setup";
import { DEFAULT_RAG_PROMPT, DEFAULT_SYSTEM_PROMPT } from "./utils/onboarding";

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
    const query = await admin
      .graphql(
        `
      query {
        shop {
          name
          email
        }
      }
    `,
      )
      .catch((e) => {
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

    const userCredentialsResponse = await fetch(
      `${getTrieveBaseUrlEnv()}/api/auth/create_api_only_user`,
      {
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
      },
    ).catch((e) => {
      console.error(e);
      throw e;
    });
    const userCredentials =
      (await userCredentialsResponse.json()) as CreateApiUserResponse;

    if (!userCredentials.api_key) {
      console.error(
        "Shopify secret key is not set",
        process.env.SHOPIFY_SECRET_KEY,
      );
      throw new Error(
        "No API key returned from create_api_only_user: " +
          JSON.stringify(userCredentials),
      );
    }

    key = await prisma.apiKey
      .upsert({
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
        },
      })
      .catch((e) => {
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
            SYSTEM_PROMPT: DEFAULT_SYSTEM_PROMPT,
            RAG_PROMPT: DEFAULT_RAG_PROMPT,
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
    throw new Response(JSON.stringify({ message: "No dataset selected" }), {
      headers: {
        "Content-Type": "application/json; charset=utf-8",
      },
      status: 401,
    });
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

export const useTrieveServer = async (request: Request) => {
  const key = await validateTrieveAuth(request);
  return { key, trieve: sdkFromKey(key) };
};
