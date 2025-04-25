import type { ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import { ShopifyPlanChangePayload } from "trieve-ts-sdk";
import db from "../db.server";
import { sdkFromKey, validateTrieveAuthWehbook } from "app/auth";

async function hashString(str: string) {
  const textEncoder = new TextEncoder();
  const data = textEncoder.encode(str);
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
  return hashHex;
}

export const action = async ({ request }: ActionFunctionArgs) => {
  const { admin, shop, payload, topic } = await authenticate.webhook(request);
  const key = await validateTrieveAuthWehbook(shop);
  const trieve = sdkFromKey(key);

  console.log(`Received ${topic} webhook for ${shop}`);
  const organization_id = await db.apiKey.findFirst({
    where: {
      shop,
    },
  });
  if (!organization_id) {
    return new Response("Organization not found", { status: 404 });
  }

  let data;
  try {
    data = await admin?.graphql(
      `
      query {
        currentAppInstallation {
          activeSubscriptions {
            currentPeriodEnd
          }
        }
      }
    `,
    );
  } catch (error) {
    console.error("Failed to fetch subscription data:", error);
    data = null;
  }

  const trievePayload: ShopifyPlanChangePayload = {
    organization_id: organization_id?.organizationId,
    idempotency_key: await hashString(
      `${payload.app_subscription.updated_at}-${payload.app_subscription.admin_graphql_api_id}`,
    ),
    shopify_plan: {
      handle: payload.app_subscription.name
        .replace("\n", "")
        .replace(/ /g, "-")
        .toLowerCase(),
      status: payload.app_subscription.status,
      current_period_end:
        (await data?.json())?.data.currentAppInstallation
          .activeSubscriptions?.[0]?.currentPeriodEnd ?? undefined,
    },
  };

  await trieve.handleShopifyPlanChange(
    trievePayload,
    process.env.SHOPIFY_SECRET_KEY || "",
  );

  return new Response();
};
