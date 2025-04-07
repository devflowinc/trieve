import { TrieveKey } from "app/types";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { ShopifyCustomer } from "trieve-ts-sdk";

export async function trackUserLinked(shopifyCustomer: ShopifyCustomer, key: TrieveKey) {
  console.log("linked to user");
  await fetch(`${getTrieveBaseUrlEnv()}/api/shopify/link`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${key.key}`,
      "Content-Type": "application/json",
      "TR-Organization": key.organizationId,
    },
    body: JSON.stringify(shopifyCustomer),
  }).catch((e) => {
    console.error(`Error sending account link to Trieve: ${e}`);
  });
}
