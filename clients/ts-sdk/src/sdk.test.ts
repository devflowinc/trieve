// sum.test.js
import {
  expect,
  test,
  describe,
  beforeEach,
  beforeAll,
  expectTypeOf,
} from "vitest";
import { TrieveSDK } from "./sdk";
import {
  CTRAnalyticsResponse,
  RAGAnalyticsResponse,
  RecommendationAnalytics,
  RecommendationAnalyticsResponse,
  RecommendResponseTypes,
} from "./types.gen";

describe("SDK Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = new TrieveSDK({
      apiKey: "tr-mKHF9sstPHQHcCbh6Qk6Uw54hx7uwDGU",
      datasetId: "c04c43d9-382d-4815-810d-b776904a7390",
    });
  });
  test("search", async () => {
    const data = await trieve.search({
      query: "one",
      search_type: "hybrid",
    });

    expect(data?.chunks).toHaveLength(10);
  });
  test("ctr analytics get", async () => {
    const data = await trieve.getCTRAnalytics({
      type: "search_ctr_metrics",
    });

    expectTypeOf(data).toEqualTypeOf<CTRAnalyticsResponse>();
  });
  test("rag analytics get", async () => {
    const data = await trieve.getRagAnalytics({
      type: "rag_queries",
    });

    expectTypeOf(data).toEqualTypeOf<RAGAnalyticsResponse>();
  });
  test("recommendation analytics get", async () => {
    const data = await trieve.getRecommendationAnalytics({
      type: "low_confidence_recommendations",
    });
    console.log(data);

    expectTypeOf(data).toEqualTypeOf<RecommendationAnalyticsResponse>();
  });
});
