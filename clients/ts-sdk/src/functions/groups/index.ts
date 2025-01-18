/**
 * This includes all the functions you can use to communicate with our groups API
 *
 * @module Group Methods
 */

import {
  AddChunkToGroupReqPayload,
  CreateChunkGroupReqPayloadEnum,
  DeleteChunkGroupData,
  DeleteGroupByTrackingIdData,
  GetChunkGroupData,
  GetChunksInGroupByTrackingIdData,
  GetChunksInGroupData,
  GetGroupByTrackingIdData,
  GetGroupsForChunksReqPayload,
  GetGroupsForDatasetData,
  RecommendGroupsReqPayload,
  RecommendGroupsResponseBody,
  RemoveChunkFromGroupReqPayload,
  SearchOverGroupsReqPayload,
  SearchOverGroupsResponseBody,
  SearchWithinGroupReqPayload,
  SearchWithinGroupResponseBody,
  UpdateChunkGroupReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

/**
 * Create new chunk_group(s). This is a way to group chunks together. If you try to create a chunk_group with the same tracking_id as an existing chunk_group, this operation will fail. Only 1000 chunk groups can be created at a time. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.createChunkGroup({
  description: "All versions and colorways of the oversized t-shirt",
  metadata: {
    color: "black",
    size: "large",
  },
  name: "Versions of Oversized T-Shirt",
  tag_set: ["tshirt", "oversized", "clothing"],
  tracking_id: "SNOVERSIZEDTSHIRT",
  upsert_by_tracking_id: false,
});
 * ```
 */
export async function createChunkGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<CreateChunkGroupReqPayloadEnum, "datasetId">
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch("/api/chunk_group", "post", {
    data,
    datasetId: this.datasetId,
  });
}

/**
 * This function allows you to get groups as results instead of chunks. Each group returned will have the matching chunks sorted by similarity within the group. This is useful for when you want to get groups of chunks which are similar to the search query. If choosing hybrid search, the results will be re-ranked using scores from a cross encoder model. Compatible with semantic, fulltext, or hybrid search modes.
 * 
 * Example:
 * ```js
 *const data = await trieve.searchOverGroups({
  query: "a query",
});
 * ```
 */
export async function searchOverGroups(
  /** @hidden */
  this: TrieveSDK,
  data: SearchOverGroupsReqPayload,
  signal?: AbortSignal,
  parseHeaders?: (headers: Record<string, string>) => void
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/group_oriented_search",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal,
    parseHeaders
  ) as Promise<SearchOverGroupsResponseBody>;
}

/**
 * This function allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. If choosing hybrid search, the results will be re-ranked using scores from a cross encoder model.
 * 
 * Example:
 * ```js
 *const data = await trieve.searchInGroup({
  query: "a query",
});
 * ```
 */
export async function searchInGroup(
  /** @hidden */
  this: TrieveSDK,
  data: SearchWithinGroupReqPayload,
  signal?: AbortSignal,
  parseHeaders?: (headers: Record<string, string>) => void
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/search",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal,
    parseHeaders
  ) as Promise<SearchWithinGroupResponseBody>;
}

/**
 * Function to get recommended groups. This route will return groups which are similar to the groups in the request body. You must provide at least one positive group id or group tracking id.
 * 
 * Example:
 * ```js
 *const data = await trieve.recommendedGroups({
  positive_group_ids: ["3c90c3cc-0d44-4b50-8888-8dd25736052a"],
});
 * ```
 */
export async function recommendedGroups(
  /** @hidden */
  this: TrieveSDK,
  data: RecommendGroupsReqPayload,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/recommend",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  ) as Promise<RecommendGroupsResponseBody>;
}

/**
 * Update a chunk_group. One of group_id or tracking_id must be provided. If you try to change the tracking_id to one that already exists, this operation will fail. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.updateGroup({
  group_id: "3c90c3cc-0d44-4b50-8888-8dd25736052a",
});
 * ```
 */
export async function updateGroup(
  /** @hidden */
  this: TrieveSDK,
  data: UpdateChunkGroupReqPayload,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group",
    "put",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Add a chunk to a group. One of chunk_id or chunk_tracking_id must be provided. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.addChunkToGroup({
  chunk_id: "3c90c3cc-0d44-4b50-8888-8dd25736052a",
});
 * ```
 */
export async function addChunkToGroup(
  /** @hidden */
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { group_id: string },
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/chunk/{group_id}",
    "post",
    {
      data,
      groupId: data.group_id,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Remove a chunk from a group. Auth’ed user or api key must be an admin or owner of the dataset’s organization to remove a chunk from a group.
 * 
 * Example:
 * ```js
 *const data = await trieve.removeChunkFromGroup({
  chunk_id: "3c90c3cc-0d44-4b50-8888-8dd25736052a",
  groupId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function removeChunkFromGroup(
  /** @hidden */
  this: TrieveSDK,
  data: RemoveChunkFromGroupReqPayload & { group_id: string },
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/chunk/{group_id}",
    "delete",
    {
      data,
      groupId: data.group_id,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Function to get the groups that a chunk is in.
 * 
 * Example:
 * ```js
 *const data = await trieve.getGroupsForChunks({
  chunk_ids: ["3c90c3cc-0d44-4b50-8888-8dd25736052a"],
});
 * ```
 */
export async function getGroupsForChunks(
  /** @hidden */
  this: TrieveSDK,
  data: GetGroupsForChunksReqPayload,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/chunks",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Function to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Support for custom page size is coming soon. Page is 1-indexed.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunksGroupByTrackingId({
  page: 1,
  groupTrackingId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function getChunksGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunksInGroupByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{group_tracking_id}/{page}",
    "get",
    {
      ...data,
      xApiVersion: data.xApiVersion || "V2",

      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Fetch the group with the given tracking id. get_group_by_tracking_id
 * 
 * Example:
 * ```js
 *const data = await trieve.getGroupByTrackingId({
  trackingId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function getGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetGroupByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Add a chunk to a group by tracking id. One of chunk_id or chunk_tracking_id must be provided. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.addChunkToGroupByTrackingId({
  tracking_id: "3c90c3cc-1d76-27198-8888-8dd25736052a"
  chunk_tracking_id: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function addChunkToGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { tracking_id: string },
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "post",
    {
      data,
      datasetId: this.datasetId,
      trackingId: data.tracking_id,
    },
    signal
  );
}

/**
 * Delete a chunk_group with the given tracking id. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteGroupByTrackingId({
  tracking_id: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function deleteGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: DeleteGroupByTrackingIdData & { tracking_id: string },
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "delete",
    {
      ...data,
      trackingId: data.tracking_id,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Fetch the group with the given id. get_group
 * 
 * Example:
 * ```js
 *const data = await trieve.getGroup({
  groupId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function getGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunkGroupData, "trDataset">,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/{group_id}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * This will delete a chunk_group. If you set delete_chunks to true, it will also delete the chunks within the group. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteGroup({
  groupId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function deleteGroup(
  /** @hidden */
  this: TrieveSDK,
  data: DeleteChunkGroupData,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/{group_id}",
    "delete",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Get all chunks for a group. The response is paginated, with each page containing 10 chunks. Page is 1-indexed.
 * 
 * Example:
 * ```js
 *const data = await trieve.getChunksInGroup({
  groupId: "3c90c3cc-1d76-27198-8888-8dd25736052a",
  page: 1
});
 * ```
 */
export async function getChunksInGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunksInGroupData, "trDataset">,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/chunk_group/{group_id}/{page}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Fetch the groups which belong to a dataset specified by its id.
 * 
 * Example:
 * ```js
 *const data = await trieve.getGroupsForDataset({
  page: 1
});
 * ```
 */
export async function getGroupsForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<Omit<GetGroupsForDatasetData, "datasetId">, "trDataset">,
  signal?: AbortSignal
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return this.trieve.fetch(
    "/api/dataset/groups/{dataset_id}/{page}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}
