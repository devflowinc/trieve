import { register } from "@shopify/web-pixels-extension";
import { TrieveSDK } from "trieve-ts-sdk";

register(async ({ analytics, browser, init, settings }) => {
  // Bootstrap and insert pixel script tag here
  const apiKey = settings.apiKey;
  const datasetId = settings.datasetId;

  let fingerprint = await browser.localStorage.getItem("trieve-fingerprint").catch((_) => {
    return null
  });
  const trieveSDK = new TrieveSDK({
    apiKey,
    datasetId,
    baseUrl: "https://api.trieve.ai"
  });

  // Sample subscribe to page view
  analytics.subscribe('checkout_started', async (event) => {
    console.log('Checkout started', event);
    const items = event.data.checkout.lineItems.map((item) => {
      return {
        tracking_id: item.variant?.id?.toString() ?? "",
        revenue: item.finalLinePrice.amount,
      };
    });

    const lastMessage = JSON.parse(
      await browser.localStorage.getItem("lastMessage").catch((_) => { return null; }) ?? "{}",
    );
    let requestId = "00000000-0000-0000-0000-000000000000";
    for (const id in lastMessage) {
      const storedItems = lastMessage[id];
      if (
        storedItems.some((item: any) =>
          items.map((i: any) => i.tracking_id).includes(item),
        )
      ) {
        requestId = id;
        break;
      }
    }
    console.log('Sending checkout started event', {
      event_name: `site-checkout`,
      event_type: "purchase",
      items,
      is_conversion: true,
      user_id: fingerprint,
      request: {
        request_id: requestId,
        request_type: "rag",
      },
    });

    trieveSDK.sendAnalyticsEvent(
      {
        event_name: `site-checkout`,
        event_type: "purchase",
        items,
        is_conversion: true,
        user_id: fingerprint,
        request: {
          request_id: requestId,
          request_type: "rag",
        },
      }
    );
  });

  analytics.subscribe('checkout_completed', async (event) => {
    console.log('Checkout completed', event);
    const items = event.data.checkout.lineItems.map((item) => {
      return {
        tracking_id: item.variant?.id?.toString() ?? "",
        revenue: item.finalLinePrice.amount,
      };
    });

    const lastMessage: { [key: string]: string[] } = JSON.parse(
      await browser.localStorage.getItem("lastMessage").catch((_) => { return null; }) ?? "{}",
    );
    let requestId = "00000000-0000-0000-0000-000000000000";
    for (const id in lastMessage) {
      const storedItems = lastMessage[id];
      if (
        storedItems.some((item: string) =>
          items.map((i) => i.tracking_id).includes(item),
        )
      ) {
        requestId = id;
        break;
      }
    }
    console.log('Sending checkout completed event', {
      event_name: `site-checkout_end`,
      event_type: "purchase",
      items,
      is_conversion: true,
      user_id: fingerprint,
    });

    trieveSDK.sendAnalyticsEvent(
      {
        event_name: `site-checkout_end`,
        event_type: "purchase",
        items,
        is_conversion: true,
        user_id: fingerprint,
        request: {
          request_id: requestId,
          request_type: "rag",
        },
      }
    );
  });

  analytics.subscribe('product_added_to_cart', async (event) => {
    console.log('Product added to cart', event);

    const cart_item = event.data.cartLine?.merchandise?.id?.toString() ?? "";

    const lastMessage: { [key: string]: string[] } = JSON.parse(
      await browser.localStorage.getItem("lastMessage").catch((_) => { return null; }) ?? "{}",
    );
    let requestId = "00000000-0000-0000-0000-000000000000";
    for (const id in lastMessage) {
      const storedItems = lastMessage[id];
      if (
        storedItems.some((item: string) => item === cart_item)
      ) {
        requestId = id;
        break;
      }
    }
    console.log('Sending product added to cart event', {
      event_name: `site-add_to_cart`,
      event_type: "add_to_cart",
      items: [cart_item],
      user_id: fingerprint,
      request: {
        request_id: requestId,
        request_type: "rag",
      },
    });

    trieveSDK.sendAnalyticsEvent(
      {
        event_name: `site-add_to_cart`,
        event_type: "add_to_cart",
        items: [cart_item],
        user_id: fingerprint,
        request: {
          request_id: requestId,
          request_type: "rag",
        },
      },
    );
  });
});
