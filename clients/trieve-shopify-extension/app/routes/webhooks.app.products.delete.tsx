import { type ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import db from "../db.server";
import { TrieveKey } from "app/types";
import { deleteChunkFromTrieve } from "app/processors/getProducts";
import { getTrieveBaseUrlEnv } from "app/env.server";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { payload, topic, shop } = await authenticate.webhook(request);
  console.log(`Received ${topic} webhook for ${shop}`);

  const current = payload as { id: string };
  const apiKey = await db.apiKey.findFirst({
    where: { shop: `https://${shop}` },
  });

  if (!apiKey) {
    console.error(`No API key found for ${shop}`);
    return new Response();
  }

  const trieveKey: TrieveKey & { [key: string]: any } = {
    createdAt: new Date(apiKey.createdAt).toISOString(),
    id: apiKey.id,
    key: apiKey.key,
    organizationId: apiKey.organizationId,
    currentDatasetId: apiKey.currentDatasetId,
  };

  deleteChunkFromTrieve(
    current.id,
    trieveKey,
    trieveKey.currentDatasetId ?? "",
    getTrieveBaseUrlEnv(),
  );

  return new Response();
};
