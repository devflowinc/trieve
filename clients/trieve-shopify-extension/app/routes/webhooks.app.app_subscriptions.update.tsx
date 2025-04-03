import type { ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";
import { useAppBridge } from "@shopify/app-bridge-react";
import { ShopifyPlanChangePayload } from "trieve-ts-sdk";
import db from "../db.server";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { shop, payload, topic } = await authenticate.webhook(request);

  console.log(`Received ${topic} webhook for ${shop}`);
  const organization_id = await db.apiKey.findFirst({
    where: {
      shop: `https://${shop}`,
    }
  })
  if (!organization_id) {
    return new Response("Organization not found", { status: 404 });
  }

  const trievePayload: ShopifyPlanChangePayload = {
    organization_id: organization_id?.organizationId,
    session_token: process.env.SHOPIFY_SECRET_KEY || "",
    shopify_plan: {
      name: payload.app_subscription.name,
      handle: payload.app_subscription.name,
      status: payload.app_subscription.status,
    },
  };

  console.log(
    trievePayload
  )

  return new Response();
};
