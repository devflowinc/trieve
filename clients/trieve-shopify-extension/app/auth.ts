import { LoaderFunctionArgs } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey, TrieveKey } from "./types";
import { TrieveSDK } from "trieve-ts-sdk";
import { getTrieveBaseUrlEnv } from "./env.server";

export const validateTrieveAuth = async <S extends boolean = true>(
  request: LoaderFunctionArgs["request"],
  strict: S = true as S,
): Promise<S extends true ? StrongTrieveKey : TrieveKey> => {
  let { sessionToken } = await authenticate.admin(request);

  const key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });

  if (!key) {
    throw new Response(
      JSON.stringify({ message: "No key matching the current user" }),
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
    userId: key.userId,
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
