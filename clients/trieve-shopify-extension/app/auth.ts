import { LoaderFunctionArgs } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey, TrieveKey } from "./types";
import { TrieveSDK, CreateApiUserResponse } from "trieve-ts-sdk";
import { getTrieveBaseUrlEnv } from "./env.server";

export const validateTrieveAuth = async <S extends boolean = true>(
  request: LoaderFunctionArgs["request"],
  strict: S = true as S,
): Promise<S extends true ? StrongTrieveKey : TrieveKey> => {
  let { admin, session } = await authenticate.admin(request);

  let key = await prisma.apiKey.findFirst({
    where: {
      shop: (session.shop as string) ?? "",
    },
  });

  if (!key) {
    const query = await admin.graphql(
      `
      query {
        shop {
          name
          email
        }
      }
    `);

    const queryResponse = await query?.json();

    const shop_name = queryResponse?.data?.shop?.name ?? "";
    const shop_email = queryResponse?.data?.shop?.email ?? "";

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
    });
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
