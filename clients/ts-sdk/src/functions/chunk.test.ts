import { expect, test, describe, beforeAll, expectTypeOf } from "vitest";
import { TrieveSDK } from "../sdk";
import {
  ChunkReturnTypes,
  CountChunkQueryResponseBody,
  CTRAnalyticsResponse,
  RAGAnalyticsResponse,
  RecommendationAnalyticsResponse,
  RecommendResponseTypes,
  ReturnQueuedChunk,
  ScrollChunksResponseBody,
  SearchResponseBody,
  SearchResponseTypes,
} from "../types.gen";

const tracking_id = "B08569DD46";
const id = "7d5ef532-80e3-4978-a174-eb99960fdc9d";
const exampleChunkHTML = `Price: $25
Brand: Whole Foods Market
Product Name: WHOLE FOODS MARKET Organic Chocolate Truffles, 8.8 OZ
Brought to you by Whole Foods Market.  When it comes to innovative flavors and products sourced from artisans and producers around the world, the Whole Foods Market brand has you covered. Amazing products, exceptional ingredients, no compromises.;Limited Edition ~ Get yours while supplies last!;Made according to an old family recipe by one of France’s leading chocolatiers, our organic truffles are rich and darkly delicious.;They’re an exceptional midday treat served with tea or espresso and a perfectly simple and satisfying finish to any evening meal.;Product of France;Low-Sodium;Vegetarian;USDA Certified Organic;QAI Certified Organic - If It's Organic It's Non-GMO;Product Type: GROCERY
Country: US
Marketplace: WholeFoods
Domain: wholefoodsmarket.com`;

describe("Chunk Methods Test", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = new TrieveSDK({
      apiKey: "tr-mKHF9sstPHQHcCbh6Qk6Uw54hx7uwDGU",
      datasetId: "6cba9148-9cbb-417a-a955-93ea749ef27c",
    });
  });
  test("search", async () => {
    const data = await trieve.search({
      query: "one",
      search_type: "hybrid",
    });

    expectTypeOf(data).toEqualTypeOf<SearchResponseBody>();
  });

  test("create chunk", async () => {
    const data = await trieve.createChunk({
      chunk_html: "test",
    });

    expectTypeOf(data).toEqualTypeOf<ReturnQueuedChunk>();
  });
  test("autocomplete", async () => {
    const data = await trieve.autocomplete({
      query: "test",
      search_type: "fulltext",
    });

    expectTypeOf(data).toEqualTypeOf<SearchResponseTypes>();
  });

  test("getRecommendedChunks", async () => {
    const data = await trieve.getRecommendedChunks({
      positive_chunk_ids: [id],
    });

    expectTypeOf(data).toEqualTypeOf<RecommendResponseTypes>();
  });

  test("ragOnChunk", async () => {
    const data = await trieve.ragOnChunk({
      chunk_ids: [id],
      prev_messages: [
        {
          content: "hello",
          role: "user",
        },
      ],
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("countChunksAboveThreshold", async () => {
    const data = await trieve.countChunksAboveThreshold({
      limit: 10,
      query: "test",
      search_type: "bm25",
    });

    expectTypeOf(data).toEqualTypeOf<CountChunkQueryResponseBody>();
  });

  test("scroll", async () => {
    const data = await trieve.scroll({});

    expectTypeOf(data).toEqualTypeOf<ScrollChunksResponseBody>();
  });
  test("updateChunk", async () => {
    const data = await trieve.updateChunk({
      tracking_id: tracking_id,
      chunk_html: exampleChunkHTML,
    });
    expectTypeOf(data).toBeVoid();
  });
  test("updateChunkByTrackingId", async () => {
    const data = await trieve.updateChunkByTrackingId({
      tracking_id: tracking_id,
      chunk_html: exampleChunkHTML,
    });

    expectTypeOf(data).toBeVoid();
  });
  test("getChunksByIds", async () => {
    const data = await trieve.getChunksByIds({
      ids: [id],
    });

    expectTypeOf(data).toEqualTypeOf<ChunkReturnTypes[]>();
  });

  test("getChunksByTrackingIds", async () => {
    const data = await trieve.getChunksByTrackingIds({
      tracking_ids: [tracking_id],
    });

    expectTypeOf(data).toEqualTypeOf<ChunkReturnTypes[]>();
  });
});
