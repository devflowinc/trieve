/**
 * This includes all the functions you can use to communicate with our chunks API
 *
 * @module Chunk Methods
 */

import {
  AutocompleteReqPayload,
  CountChunksReqPayload,
  CreateChunkReqPayloadEnum,
  DeleteChunkByTrackingIdData,
  DeleteChunkData,
  GenerateOffChunksReqPayload,
  GetChunkByIdData,
  GetChunkByTrackingIdData,
  GetChunksData,
  GetTrackingChunksData,
  RecommendChunksRequest,
  ScrollChunksReqPayload,
  SearchChunksReqPayload,
  SearchResponseBody,
  SuggestedQueriesReqPayload,
  UpdateChunkByTrackingIdData,
  UpdateChunkReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

export async function search(
  /** @hidden */
  this: TrieveSDK,
  props: SearchChunksReqPayload,
  signal?: AbortSignal
) {
  const searchResults = (await this.trieve.fetch(
    "/api/chunk/search",
    "post",
    {
      xApiVersion: "V2",
      data: props,
      datasetId: this.datasetId,
    },
    signal
  )) as SearchResponseBody;

  return searchResults;
}
export async function createChunk(
  /** @hidden */
  this: TrieveSDK,
  props: CreateChunkReqPayloadEnum,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}
export async function autocomplete(
  /** @hidden */
  this: TrieveSDK,
  props: AutocompleteReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/autocomplete",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getRecommendedChunks(
  /** @hidden */
  this: TrieveSDK,
  props: RecommendChunksRequest,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/recommend",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function ragOnChunk(
  /** @hidden */
  this: TrieveSDK,
  props: GenerateOffChunksReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/generate",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function suggestedQueries(
  /** @hidden */
  this: TrieveSDK,
  props: SuggestedQueriesReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/suggestions",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function countChunksAboveThreshold(
  /** @hidden */
  this: TrieveSDK,
  props: CountChunksReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/count",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function scroll(
  /** @hidden */
  this: TrieveSDK,
  props: ScrollChunksReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunks/scroll",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}
export async function updateChunk(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk",
    "put",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}
export async function updateChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkByTrackingIdData,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/tracking_id/update",
    "put",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/tracking_id/{tracking_id}",
    "get",
    {
      trackingId: props.trackingId,
      datasetId: this.datasetId,
      xApiVersion: props.xApiVersion ?? "V2",
    },
    signal
  );
}

export async function deleteChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<DeleteChunkByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/tracking_id/{tracking_id}",
    "delete",
    {
      trackingId: props.trackingId,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByIdData, "trDataset">,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/{chunk_id}",
    "get",
    {
      chunkId: props.chunkId,
      xApiVersion: props.xApiVersion ?? "V2",
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function deleteChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: DeleteChunkData,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk/{chunk_id}",
    "delete",
    {
      chunkId: props.chunkId,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getChunksByIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetChunksData,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunks",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getChunksByTrackingIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetTrackingChunksData,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunks/tracking",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal
  );
}
