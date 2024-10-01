import { beforeAll, describe, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import {
  CTRAnalyticsResponse,
  GetEventsResponseBody,
  RAGAnalyticsResponse,
  RecommendationAnalyticsResponse,
  SearchAnalyticsResponse,
  TopDatasetsResponse,
} from "../../types.gen";
import { CHUNK_EXAMPLE_ID, TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

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
  test("rateRagQuery", async () => {
    const data = await trieve.rateRagQuery({
      rating: 1,
      query_id: "0dc64b20-b565-478b-b259-de4cd8f8688a",
    });

    expectTypeOf(data).toBeVoid();
  });
  test("getTopDatasets", async () => {
    const data = await trieve.getTopDatasets({
      organizationId: "de73679c-707f-4fc2-853e-994c910d944c",
      type: "rag",
    });

    expectTypeOf(data).toEqualTypeOf<TopDatasetsResponse[]>();
  });
});
