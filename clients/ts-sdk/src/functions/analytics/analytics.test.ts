import { beforeAll, describe, expectTypeOf, test } from "vitest";
import { TrieveSDK } from "../../sdk";
import {
  CTRAnalyticsResponse,
  RAGAnalyticsResponse,
  RecommendationAnalyticsResponse,
  SearchAnalyticsResponse,
} from "../../types.gen";
import { CHUNK_EXAMPLE_ID, TRIEVE } from "../../../__tests__/constants";

describe("Analytics Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("ctr analytics get", async () => {
    const data = await trieve.getCTRAnalytics({
      type: "search_ctr_metrics",
    });

    expectTypeOf(data).toEqualTypeOf<CTRAnalyticsResponse>();
  });
  test("sendCTRAnalytics", async () => {
    const data = await trieve.sendCTRAnalytics({
      clicked_chunk_id: CHUNK_EXAMPLE_ID,
      ctr_type: "search",
      metadata: "query",
      position: 123,
      request_id: CHUNK_EXAMPLE_ID,
    });

    expectTypeOf(data).toBeVoid();
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

    expectTypeOf(data).toEqualTypeOf<RecommendationAnalyticsResponse>();
  });
  test("getSearchAnalytics", async () => {
    const data = await trieve.getSearchAnalytics({
      type: "count_queries",
    });

    expectTypeOf(data).toEqualTypeOf<SearchAnalyticsResponse>();
  });
});
