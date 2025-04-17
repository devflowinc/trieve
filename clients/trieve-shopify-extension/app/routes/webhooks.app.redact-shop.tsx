import { ActionFunctionArgs } from "@remix-run/node";
import db from "../db.server";
import { TrieveKey } from "app/types";
import { sdkFromKey } from "app/auth";

export const action = async ({ request }: ActionFunctionArgs) => {

  const json = await request.json();

  const shop = json.shop_domain;
  if (!shop) {
    return new Response();
  }

  const apiKey = await db.apiKey.findFirst({
    where: { shop: json.shop_domain },
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

  const trieve = sdkFromKey(trieveKey);

  trieve.deleteDataset(trieveKey.currentDatasetId ?? "");

  await db.apiKey.delete({
    where: { id: trieveKey.id },
  });

  return new Response();
}
