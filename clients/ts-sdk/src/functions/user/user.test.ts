import { beforeAll, describe, expect, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { CreateApiKeyResponse, ReturnQueuedChunk } from "../../types.gen";
import { TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("User Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });

  test("create an api key and verify it works", async () => {
    const apiKeyResponse = await trieve.createUserApiKey({
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

    trieve.deleteUserApiKey(apiKeyResponse.api_key);
  });

  test("create an expired api key and verify it does not work", async () => {
    const apiKeyResponse = await trieve.createUserApiKey({
      expires_at: new Date(new Date().setDate(new Date().getDate() - 2))
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

    trieve.deleteUserApiKey(apiKeyResponse.api_key);
  });
});
