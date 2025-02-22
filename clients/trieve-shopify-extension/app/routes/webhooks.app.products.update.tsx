import { data, type ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import db from "../db.server";
import { ProductWebhook, TrieveKey } from "app/types";
import {
  chunk_to_size,
  createChunkFromProductWebhook,
  sendChunksToTrieve,
} from "app/processors/getProducts";
import { ChunkReqPayload } from "trieve-ts-sdk";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { payload, session, topic, shop } =
    await authenticate.webhook(request);
  console.log(`Received ${topic} webhook for ${shop}`);

  const current = payload as ProductWebhook;
  const apiKey = await db.apiKey.findFirst({
    where: { shop: `https://${shop}` },
  });

  if (!apiKey) {
    console.error(`No API key found for ${shop}`);
    return new Response();
  }

  const trieveKey: TrieveKey = {
    createdAt: new Date(apiKey.createdAt).toISOString(),
    id: apiKey.id,
    key: apiKey.key,
    organizationId: apiKey.organizationId,
    currentDatasetId: apiKey.currentDatasetId,
    userId: apiKey.userId,
  };

  const dataChunks: ChunkReqPayload[] = current.variants.map((variant) =>
    createChunkFromProductWebhook(current, variant, `https://${session?.shop}`),
  );

  for (const batch of chunk_to_size(dataChunks, 120)) {
    sendChunksToTrieve(batch, trieveKey, trieveKey.currentDatasetId ?? "");
  }

  return new Response();
};
