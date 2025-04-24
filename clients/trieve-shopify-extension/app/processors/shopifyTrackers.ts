import { ShopifyCustomerEvent } from "trieve-ts-sdk";

export async function trackCustomerEvent(
  trieveUrl: string,
  shopifyCustomerEvent: ShopifyCustomerEvent,
  organizationId: string,
  apiKey: string,
) {
  await fetch(`${trieveUrl}/api/shopify/user_event`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
      "TR-Organization": organizationId,
    },
    body: JSON.stringify(shopifyCustomerEvent),
  }).catch((e) => {
    console.error(`Error sending account link to Trieve: ${e}`);
  });
}
