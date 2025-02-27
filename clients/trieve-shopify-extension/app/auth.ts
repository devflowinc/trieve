import { LoaderFunctionArgs, json } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey, TrieveKey } from "./types";
import { TrieveSDK } from "trieve-ts-sdk";

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
    throw json({ message: "No Key" }, 401);
  }

  if (strict && !key.currentDatasetId) {
    throw json({ message: "No dataset selected" }, 401);
  }

  return {
    id: key.id,
    key: key.key,
    organizationId: key.organizationId,
    currentDatasetId: key.currentDatasetId,
    userId: key.userId,
  } as S extends true ? StrongTrieveKey : TrieveKey;
};

export const validateTrieveDataset = async (
  request: LoaderFunctionArgs["request"],
): Promise<TrieveKey> => {
  const result = await validateTrieveAuth(request);
  if (!result.currentDatasetId) {
    throw json({ message: "No dataset selected" }, 401);
  }
  return result;
};

export const sdkFromKey = (key: TrieveKey): TrieveSDK => {
  const trieve = new TrieveSDK({
    baseUrl: "https://api.trieve.ai",
    apiKey: key.key,
    datasetId: key.currentDatasetId ? key.currentDatasetId : undefined,
    organizationId: key.organizationId,
    omitCredentials: true,
  });

  return trieve;
};
