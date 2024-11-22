import { describe, beforeAll, test, expectTypeOf, expect } from "vitest";
import { TRIEVE } from "../../__tests__/constants";
import { TrieveSDK } from "../../sdk";
import { CreateApiKeyResponse, ReturnQueuedChunk } from "../../types.gen";

describe("Organization Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });

  test("create an api key and verify it works", async () => {
    const apiKeyResponse = await trieve.createOrganizationApiKey({
      role: 1,
      name: "test suite key",
    });

    expectTypeOf(apiKeyResponse).toEqualTypeOf<CreateApiKeyResponse>();

    const newTrieve = new TrieveSDK({
      apiKey: apiKeyResponse.api_key,
      datasetId: trieve.datasetId,
    });

    const queuedChunk = await newTrieve.createChunk({
      chunk_html: "testing hello world",
      tracking_id: "1234",
      tag_set: ["test"],
    });

    expectTypeOf(queuedChunk).toEqualTypeOf<ReturnQueuedChunk>();

    newTrieve.deleteChunkByTrackingId({
      trackingId: "1234",
    });
  });

  test("create an expired api key and verify it does not work", async () => {
    const apiKeyResponse = await trieve.createOrganizationApiKey({
      expires_at: new Date(new Date().setDate(new Date().getDate() - 1))
        .toISOString()
        .slice(0, 19)
        .replace("T", " "),
      role: 1,
      name: "test suite key",
    });

    expectTypeOf(apiKeyResponse).toEqualTypeOf<CreateApiKeyResponse>();

    let errorOccurred = false;

    const newTrieve = new TrieveSDK({
      apiKey: apiKeyResponse.api_key,
      datasetId: trieve.datasetId,
    });

    try {
      await newTrieve.createChunk({
        chunk_html: "testing hello world",
        tracking_id: "should_never_work",
        tag_set: ["test"],
      });

      newTrieve.deleteChunkByTrackingId({
        trackingId: "should_never_work",
      });
    } catch (e) {
      errorOccurred = true;
    }

    expect(errorOccurred).toBe(true);
  });

  test("create an api key with a filter for test and verify it excludes chunks without the tag", async () => {
    const apiKeyResponse = await trieve.createOrganizationApiKey({
      role: 1,
      name: "test suite key",
      default_params: {
        filters: {
          must: [
            {
              field: "tag_set",
              match_all: ["test"],
            },
          ],
        },
      },
    });

    expectTypeOf(apiKeyResponse).toEqualTypeOf<CreateApiKeyResponse>();

    const newTrieve = new TrieveSDK({
      apiKey: apiKeyResponse.api_key,
      datasetId: trieve.datasetId,
    });

    const queuedChunks = await newTrieve.createChunk([
      {
        chunk_html: "testing hello world",
        tracking_id: "not_test",
        tag_set: ["not_test"],
      },
      {
        chunk_html: "testing hello world",
        tracking_id: "test",
        tag_set: ["test"],
      },
    ]);

    expectTypeOf(queuedChunks).toEqualTypeOf<ReturnQueuedChunk>();

    await new Promise((r) => setTimeout(r, 10000));

    const chunksResp = await newTrieve.scroll({
      page_size: 100,
      filters: {
        must: [
          {
            field: "tag_set",
            match_all: ["not_test"],
          },
        ],
      },
    });

    for (const chunk of chunksResp.chunks) {
      expect(chunk.tag_set).toContain("test");
    }

    newTrieve.deleteChunkByTrackingId({
      trackingId: "1234",
    });
  });
});
