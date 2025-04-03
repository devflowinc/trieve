import type { ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "../shopify.server";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { shop, payload, topic, admin } = await authenticate.webhook(request);

  console.log(`Received ${topic} webhook for ${shop}`);

  const subscription = await admin?.graphql(
    `
   query {
    currentAppInstallation {
      activeSubscriptions {
        status
        name
      }
    }
   }
   `
  );

  console.log((await subscription?.json())?.data.currentAppInstallation.activeSubscriptions);

  return new Response();
};
