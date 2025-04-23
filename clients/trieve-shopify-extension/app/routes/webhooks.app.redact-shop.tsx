import { ActionFunctionArgs } from "@remix-run/node";
import db from "../db.server";
import { TrieveKey } from "app/types";
import { sdkFromKey } from "app/auth";
import { authenticate } from "app/shopify.server";

export const action = async ({ request }: ActionFunctionArgs) => {
  await authenticate.webhook(request);

  let json;
  try {
    json = await request.json();
  } catch (error) {
    console.error(error);
    return new Response();
  }

  const shop = json.shop_domain;
  if (!shop) {
    return new Response();
  }

  let apiKey;
  try {
    apiKey = await db.apiKey.findFirst({
      where: { shop: json.shop_domain },
    });
  } catch (error) {
    console.error(error);
    return new Response();
  }

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

  const trieve = sdkFromKey(trieveKey);

  trieve.deleteDataset(trieveKey.currentDatasetId ?? "");

  try {
    await db.apiKey.delete({
      where: { id: trieveKey.id },
    });
  } catch (error) {
    console.error(error);
    return new Response();
  }

  return new Response();
}
