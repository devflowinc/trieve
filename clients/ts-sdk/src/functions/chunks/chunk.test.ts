import { describe, beforeAll, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import {
  ChunkReturnTypes,
  CountChunkQueryResponseBody,
  RecommendChunksResponseBody,
  ReturnQueuedChunk,
  ScrollChunksResponseBody,
  SearchResponseBody,
} from "../../types.gen";
import {
  CHUNK_EXAMPLE_ID,
  CHUNK_EXAMPLE_TRACKING_ID,
  EXAMPLE_CHUNK_HTML,
  TRIEVE,
} from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("Chunk Methods Test", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
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

    expectTypeOf(data).toEqualTypeOf<SearchResponseBody>();
  });

  test("getRecommendedChunks", async () => {
    const data = await trieve.getRecommendedChunks({
      positive_chunk_ids: [CHUNK_EXAMPLE_ID],
    });

    expectTypeOf(data).toEqualTypeOf<RecommendChunksResponseBody>();
  });

  test("ragOnChunk", async () => {
    const data = await trieve.ragOnChunk({
      chunk_ids: [CHUNK_EXAMPLE_ID],
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
      tracking_id: CHUNK_EXAMPLE_TRACKING_ID,
      chunk_html: EXAMPLE_CHUNK_HTML,
    });
    expectTypeOf(data).toBeVoid();
  });
  test("updateChunkByTrackingId", async () => {
    const data = await trieve.updateChunkByTrackingId({
      tracking_id: CHUNK_EXAMPLE_TRACKING_ID,
      chunk_html: EXAMPLE_CHUNK_HTML,
    });

    expectTypeOf(data).toBeVoid();
  });
  test("getChunksByIds", async () => {
    const data = await trieve.getChunksByIds({
      ids: [CHUNK_EXAMPLE_ID],
    });

    expectTypeOf(data).toEqualTypeOf<ChunkReturnTypes[]>();
  });

  test("getChunksByTrackingIds", async () => {
    const data = await trieve.getChunksByTrackingIds({
      tracking_ids: [CHUNK_EXAMPLE_TRACKING_ID],
    });

    expectTypeOf(data).toEqualTypeOf<ChunkReturnTypes[]>();
  });
});
