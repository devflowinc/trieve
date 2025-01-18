/**
 * This includes all the functions you can use to communicate with our chunks API
 *
 * @module Chunk Methods
 */

import {
  AutocompleteReqPayload,
  ChunkHtmlContentReqPayload,
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
  RecommendChunksResponseBody,
  ScrollChunksReqPayload,
  SearchChunksReqPayload,
  SearchResponseBody,
  SuggestedQueriesReqPayload,
  UpdateChunkByTrackingIdData,
  UpdateChunkReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";
import { getCleanFetch } from "../message";

/**
 * Function that provides the primary search functionality for the API. It can be used to search for chunks by semantic similarity, full-text similarity, or a combination of both. Results’ chunk_html values will be modified with <b><mark> tags for sub-sentence highlighting.
 * 
 * Example:
 * ```js
 *const data = await trieve.search({
  page: 1,
  page_size: 10,
  query: "Some search query",
 });
 * ```
 */
export async function search(
  /** @hidden */
  this: TrieveSDK,
  props: SearchChunksReqPayload,
  signal?: AbortSignal,
  parseHeaders?: (headers: Record<string, string>) => void,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }
  return this.trieve.fetch(
    "/api/chunk/search",
    "post",
    {
      xApiVersion: "V2",
      data: props,
      datasetId: this.datasetId,
    },
    signal,
    parseHeaders,
  ) as Promise<SearchResponseBody>;
}

/**
 * Function that create new chunk(s). If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint. If uploading in bulk, the maximum amount of chunks that can be uploaded at once is 120 chunks. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.createChunk({
  chunk_html: "<p>Some HTML content</p>",
  metadata: {
    key1: "value1",
    key2: "value2",
  },
});
 * ```
 */
export async function createChunk(
  /** @hidden */
  this: TrieveSDK,
  props: CreateChunkReqPayloadEnum,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Function provides the primary autocomplete functionality for the API. This prioritize prefix matching with semantic or full-text search.
 * 
 * Example:
 * ```js
 *const data = await trieve.autocomplete({
  page: 1,
  page_size: 10,
  query: "Some search query",
  search_type: "semantic",
});
 * ```
 */
export async function autocomplete(
  /** @hidden */
  this: TrieveSDK,
  props: AutocompleteReqPayload,
  signal?: AbortSignal,
  parseHeaders?: (headers: Record<string, string>) => void,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/autocomplete",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
      xApiVersion: "V2",
    },
    signal,
    parseHeaders,
  ) as Promise<SearchResponseBody>;
}

/**
 * Function that allows you to recommendations of chunks similar to the positive samples in the request and dissimilar to the negative.
 * 
 * Example:
 * ```js
 *const data = await trieve.getRecommendedChunks({
  positive_chunk_ids: [
    "3c90c3cc-0d44-4b50-8888-8dd25736052a"
  ],
});
 * ```
 */
export async function getRecommendedChunks(
  /** @hidden */
  this: TrieveSDK,
  props: RecommendChunksRequest,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/recommend",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  ) as Promise<RecommendChunksResponseBody>;
}

/**
 * This function exists as an alternative to the topic+message resource pattern where our Trieve handles chat memory. With this endpoint, the user is responsible for providing the context window and the prompt and the conversation is ephemeral.
 * 
 * Example:
 * ```js
 *const data = await trieve.ragOnChunk({
  chunk_ids: ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
  prev_messages: [
    {
      content: "How do I setup RAG with Trieve?",
      role: "user",
    },
  ],
  prompt:
    "Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:",
  stream_response: true,
});
 * ```
 */
export async function ragOnChunk(
  /** @hidden */
  this: TrieveSDK,
  props: GenerateOffChunksReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/generate",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * This function is just like ragOnChunk but it returns a reader to parse the stream easier.
 * This function exists as an alternative to the topic+message resource pattern where our Trieve handles chat memory. With this endpoint, the user is responsible for providing the context window and the prompt and the conversation is ephemeral.
 * 
 * 
 * Example:
 * ```js
 *const reader = await trieve.ragOnChunkReader({
  chunk_ids: ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
  prev_messages: [
    {
      content: "How do I setup RAG with Trieve?",
      role: "user",
    },
  ],
  prompt:
    "Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:",
  stream_response: true,
});
 * ```
 */
export async function ragOnChunkReader(
  /** @hidden */
  this: TrieveSDK,
  props: GenerateOffChunksReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  const response = await fetch(this.trieve.baseUrl + "/api/chunk/generate", {
    method: "post",
    headers: {
      "Content-Type": "application/json",
      "TR-Dataset": this.datasetId,
      Authorization: `Bearer ${this.trieve.apiKey}`,
    },
    body: JSON.stringify(props),
    signal,
  });

  const reader = response.body?.getReader();

  if (!reader) {
    throw new Error("Failed to get reader from response body");
  }

  return reader;
}

/**
 * This function is just like ragOnChunk but it returns a reader to parse the stream easier.
 * This function exists as an alternative to the topic+message resource pattern where our Trieve handles chat memory. With this endpoint, the user is responsible for providing the context window and the prompt and the conversation is ephemeral.
 * 
 * 
 * Example:
 * ```js
 *const { reader, queryId } = await trieve.ragOnChunkReader({
  chunk_ids: ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
  prev_messages: [
    {
      content: "How do I setup RAG with Trieve?",
      role: "user",
    },
  ],
  prompt:
    "Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:",
  stream_response: true,
});
 * ```
 */
export async function ragOnChunkReaderWithQueryId(
  /** @hidden */
  this: TrieveSDK,
  props: GenerateOffChunksReqPayload,
  signal?: AbortSignal,
  parseHeaders?: (headers: Record<string, string>) => void,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  const cleanFetch = getCleanFetch();
  const fetchToUse = cleanFetch ?? fetch;

  const response = await fetchToUse(
    this.trieve.baseUrl + "/api/chunk/generate",
    {
      method: "post",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": this.datasetId,
        Authorization: `Bearer ${this.trieve.apiKey}`,
      },
      body: JSON.stringify(props),
      signal,
    }
  );

  if (parseHeaders) {
    parseHeaders(Object.fromEntries(response.headers.entries()));
  }

  const reader = response.body?.getReader();

  if (!reader) {
    throw new Error("Failed to get reader from response body");
  }

  const queryId = response.headers.get("TR-QueryID");

  return {
    reader,
    queryId,
  };
}

/**
 * This function will generate 3 suggested queries based off a hybrid search using RAG with the query provided in the request body and return them as a JSON object.
 * 
 * Example:
 * ```js
 *const data = await trieve.suggestedQueries({
  query: "Some search query",
});
 * ```
 */
export async function suggestedQueries(
  /** @hidden */
  this: TrieveSDK,
  props: SuggestedQueriesReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/suggestions",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * This function can be used to determine the number of chunk results that match a search query including score threshold and filters. It may be high latency for large limits. There is a dataset configuration imposed restriction on the maximum limit value (default 10,000) which is used to prevent DDOS attacks. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.countChunksAboveThreshold({
  query: "Some search query",
  score_threshold: 0.5,
  search_type: "semantic",
});
 * ```
 */
export async function countChunksAboveThreshold(
  /** @hidden */
  this: TrieveSDK,
  props: CountChunksReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/count",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get paginated chunks from your dataset with filters and custom sorting. If sort by is not specified, the results will sort by the id’s of the chunks in ascending order. Sort by and offset_chunk_id cannot be used together; if you want to scroll with a sort by then you need to use a must_not filter with the ids you have already seen. There is a limit of 1000 id’s in a must_not filter at a time.
 * 
 * Example:
 * ```js
 *const data = await trieve.scroll({
  page_size: 10
});
 * ```
 */
export async function scroll(
  /** @hidden */
  this: TrieveSDK,
  props: ScrollChunksReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunks/scroll",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Update a chunk. If you try to change the tracking_id of the chunk to have the same tracking_id as an existing chunk, the request will fail. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.autocomplete({
  chunk_html: "<p>Some HTML content</p>",
  chunk_id: "d290f1ee-6c54-4b01-90e6-d701748f0851",
});
 * ```
 */
export async function updateChunk(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk",
    "put",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Update a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.updateChunkByTrackingId({
  chunk_html: "New text",
  tracking_id: "128ABC",
});
 * ```
 */
export async function updateChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateChunkByTrackingIdData,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/tracking_id/update",
    "put",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get a singular chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use your own id as the primary reference for a chunk.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunkByTrackingId({
  tracking_id: "128ABC",
});
 * ```
 */
export async function getChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByTrackingIdData, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/tracking_id/{tracking_id}",
    "get",
    {
      trackingId: props.trackingId,
      datasetId: this.datasetId,
      xApiVersion: props.xApiVersion ?? "V2",
    },
    signal,
  );
}

/**
 * Delete a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteChunkByTrackingId({
  tracking_id: "128ABC",
});
 * ```
 */
export async function deleteChunkByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<DeleteChunkByTrackingIdData, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/tracking_id/{tracking_id}",
    "delete",
    {
      trackingId: props.trackingId,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get a singular chunk by id.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunkById({
  chunkId: "128ABC",
});
 * ```
 */
export async function getChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: Omit<GetChunkByIdData, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/{chunk_id}",
    "get",
    {
      chunkId: props.chunkId,
      xApiVersion: props.xApiVersion ?? "V2",
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Delete a singular chunk by id.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteChunkById({
  chunkId: "128ABC",
});
 * ```
 */
export async function deleteChunkById(
  /** @hidden */
  this: TrieveSDK,
  props: DeleteChunkData,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk/{chunk_id}",
    "delete",
    {
      chunkId: props.chunkId,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get multiple chunks by multiple ids.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunksByIds( {
  ids: ["3c90c3cc-0d44-4b50-8888-8dd25736052a"],
});
 * ```
 */
export async function getChunksByIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetChunksData,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunks",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get multiple chunks by multiple tracking ids.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunksByIds( {
  tracking_ids: ["3c90c3cc-0d44-4b50-8888-8dd25736052a"],
});
 * ```
 */
export async function getChunksByTrackingIds(
  /** @hidden */
  this: TrieveSDK,
  props: GetTrackingChunksData,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunks/tracking",
    "post",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Function that splits an html string into chunks.
 * The html string will be split into chunks based on the number of characters in the string and header tags.
 *
 * Example:
 * ```js
 *const data = await trieve.splitChunkHtml({
 *    chunk_html: "<p>Some HTML content</p>",
 *});
 * ```
 */
export async function splitChunkHtml(
  /** @hidden */
  this: TrieveSDK,
  props: ChunkHtmlContentReqPayload,
  signal?: AbortSignal,
) {
  return this.trieve.fetch(
    "/api/chunk/split",
    "post",
    {
      data: props,
    },
    signal,
  );
}
