import { type ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import db from "../db.server";
import { ProductWebhook, TrieveKey } from "app/types";
import { sendChunksFromWebhook } from "app/processors/getProducts";
import { ExtendedCrawlOptions } from "app/components/settings/DatasetSettings";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { admin, payload, session, topic, shop } =
    await authenticate.webhook(request);
  console.log(`Received ${topic} webhook for ${shop}`);

  if (!session) {
    console.error(`No session found for ${shop}`);
    throw new Error("No session found");
  }

  const adminApi = buildAdminApiFetcherForServer(
    session.shop,
    session?.accessToken!,
  );

  const current = payload as ProductWebhook;

  const apiKey = await db.apiKey.findFirst({
    where: { shop: `${shop}` },
  });

  if (!apiKey) {
    console.error(`No API key found for ${shop}`);
    return new Response();
  }

  const trieveKey: TrieveKey = {
    id: apiKey.id,
    key: apiKey.key,
    organizationId: apiKey.organizationId,
    currentDatasetId: apiKey.currentDatasetId,
  };

  let crawlSettings = await db.crawlSettings.findFirst({
    where: {
      datasetId: trieveKey.currentDatasetId ?? "",
      shop: shop,
    },
  });

  if (!crawlSettings) {
    console.error(`No crawl settings found for ${shop}`);
    return new Response();
  }

  sendChunksFromWebhook(
    current,
    trieveKey,
    trieveKey.currentDatasetId ?? "",
    adminApi,
    session,
    crawlSettings.crawlSettings as ExtendedCrawlOptions,
  );

  return new Response();
};
