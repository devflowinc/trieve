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
  props: SearchChunksReqPayload
) {
  const searchResults = (await this.trieve.fetch("/api/chunk/search", "post", {
    xApiVersion: "V2",
    data: props,
    datasetId: this.datasetId,
  })) as SearchResponseBody;

  return searchResults;
}
export async function createChunk(
  /** @hidden */
  this: TrieveSDK,
  props: CreateChunkReqPayloadEnum
) {
  return this.trieve.fetch("/api/chunk", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}
export async function autocomplete(
  /** @hidden */
  this: TrieveSDK,
  props: AutocompleteReqPayload
) {
  return this.trieve.fetch("/api/chunk/autocomplete", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function getRecommendedChunks(
  /** @hidden */
  this: TrieveSDK,
  props: RecommendChunksRequest
) {
  return this.trieve.fetch("/api/chunk/recommend", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function ragOnChunk(
  /** @hidden */
  this: TrieveSDK,
  props: GenerateOffChunksReqPayload
) {
  return this.trieve.fetch("/api/chunk/generate", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function suggestedQueries(
  /** @hidden */
  this: TrieveSDK,
  props: SuggestedQueriesReqPayload
) {
  return this.trieve.fetch("/api/chunk/suggestions", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function countChunksAboveThreshold(
  /** @hidden */
  this: TrieveSDK,
  props: CountChunksReqPayload
) {
  return this.trieve.fetch("/api/chunk/count", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function scroll(
  /** @hidden */
  this: TrieveSDK,
  props: ScrollChunksReqPayload
) {
  return this.trieve.fetch("/api/chunks/scroll", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}
export async function updateChunk(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkReqPayload
) {
  return this.trieve.fetch("/api/chunk", "put", {
    data: props,
    datasetId: this.datasetId,
  });
}
export async function updateChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkByTrackingIdData
) {
  return this.trieve.fetch("/api/chunk/tracking_id/update", "put", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function getChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByTrackingIdData, "trDataset">
) {
  return this.trieve.fetch("/api/chunk/tracking_id/{tracking_id}", "get", {
    trackingId: props.trackingId,
    datasetId: this.datasetId,
    xApiVersion: props.xApiVersion ?? "V2",
  });
}

export async function deleteChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<DeleteChunkByTrackingIdData, "trDataset">
) {
  return this.trieve.fetch("/api/chunk/tracking_id/{tracking_id}", "delete", {
    trackingId: props.trackingId,
    datasetId: this.datasetId,
  });
}

export async function getChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByIdData, "trDataset">
) {
  return this.trieve.fetch("/api/chunk/{chunk_id}", "get", {
    chunkId: props.chunkId,
    xApiVersion: props.xApiVersion ?? "V2",
    datasetId: this.datasetId,
  });
}

export async function deleteChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: DeleteChunkData
) {
  return this.trieve.fetch("/api/chunk/{chunk_id}", "delete", {
    chunkId: props.chunkId,
    datasetId: this.datasetId,
  });
}

export async function getChunksByIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetChunksData
) {
  return this.trieve.fetch("/api/chunks", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}

export async function getChunksByTrackingIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetTrackingChunksData
) {
  return this.trieve.fetch("/api/chunks/tracking", "post", {
    data: props,
    datasetId: this.datasetId,
  });
}
