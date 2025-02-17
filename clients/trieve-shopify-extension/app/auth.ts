import { LoaderFunctionArgs, json } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { TrieveKey } from "./types";
import { TrieveSDK } from "trieve-ts-sdk";

export const validateTrieveAuth = async ({
  request,
}: LoaderFunctionArgs): Promise<TrieveKey> => {
  let { sessionToken } = await authenticate.admin(request);

  const key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });
  if (!key) {
    throw json({ message: "No Key" }, 401);
  }
  return {
    createdAt: new Date(key.createdAt).toISOString(),
    id: key.id,
    key: key.key,
    organizationId: key.organizationId,
    currentDatasetId: key.currentDatasetId,
    userId: key.userId,
  };
};

export const initTrieveSdk = async ({
  request,
}: LoaderFunctionArgs): Promise<TrieveSDK> => {
  let { sessionToken } = await authenticate.admin(request);

  const key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });
  if (!key) {
    throw json({ message: "No Key" }, 401);
  }

  const trieve = new TrieveSDK({
    baseUrl: "https://api.trieve.ai",
    apiKey: key.key,
    datasetId: key.currentDatasetId ? key.currentDatasetId : undefined,
    organizationId: key.organizationId,
    omitCredentials: true,
  });

  return trieve;
};
