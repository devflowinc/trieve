import { TrieveSDK } from "../sdk";

export const GROUP_EXAMPLE_ID = "460e5ee8-98bc-4fed-b4ec-68f4d6453e5f";
export const GROUP_EXAMPLE_TRACKING_ID = "1234";
export const TRIEVE = new TrieveSDK({
  apiKey: "tr-mKHF9sstPHQHcCbh6Qk6Uw54hx7uwDGU",
  datasetId: "6cba9148-9cbb-417a-a955-93ea749ef27c",
  organizationId: "de73679c-707f-4fc2-853e-994c910d944c",
});

// export const TRIEVE = new TrieveSDK({
//   baseUrl: "http://localhost:8090",
//   organizationId: "310e51a2-4bb4-4317-9661-b5ef3ca85713",
//   datasetId: "e390d3b2-7c51-4695-a3ec-4487c32220b0",
//   apiKey: "admin",
// });

export const EXAMPLE_TOPIC_ID = "f85984e1-7818-4971-b300-2f462fe1a5a2";
export const EXAMPLE_MESSAGE_ID = "48d0d2ef-3bfa-4124-8625-3c625ffa45a6";

export const CHUNK_EXAMPLE_TRACKING_ID = "B08569DD46";
export const CHUNK_EXAMPLE_ID = "7d5ef532-80e3-4978-a174-eb99960fdc9d";
export const EXAMPLE_CHUNK_HTML = `Price: $25
Brand: Whole Foods Market
Product Name: WHOLE FOODS MARKET Organic Chocolate Truffles, 8.8 OZ
Brought to you by Whole Foods Market.  When it comes to innovative flavors and products sourced from artisans and producers around the world, the Whole Foods Market brand has you covered. Amazing products, exceptional ingredients, no compromises.;Limited Edition ~ Get yours while supplies last!;Made according to an old family recipe by one of France’s leading chocolatiers, our organic truffles are rich and darkly delicious.;They’re an exceptional midday treat served with tea or espresso and a perfectly simple and satisfying finish to any evening meal.;Product of France;Low-Sodium;Vegetarian;USDA Certified Organic;QAI Certified Organic - If It's Organic It's Non-GMO;Product Type: GROCERY
Country: US
Marketplace: WholeFoods
Domain: wholefoodsmarket.com`;
export const EXAMPLE_FILE_ID = "a88bea87-7eb7-467e-84d2-01bbdfecdf11";
