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
  RemoveChunkFromGroupReqPayload,
  SearchOverGroupsReqPayload,
  SearchWithinGroupReqPayload,
  UpdateChunkGroupReqPayload,
  UpdateGroupByTrackingIDReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

export async function createChunkGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<CreateChunkGroupReqPayloadEnum, "datasetId">
) {
  return this.trieve.fetch("/api/chunk_group", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function searchOverGroups(
  /** @hidden */
  this: TrieveSDK,
  data: SearchOverGroupsReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/group_oriented_search",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function searchInGroup(
  /** @hidden */
  this: TrieveSDK,
  data: SearchWithinGroupReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/search",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function recommendedGroups(
  /** @hidden */
  this: TrieveSDK,
  data: RecommendGroupsReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/recommend",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function updateGroup(
  /** @hidden */
  this: TrieveSDK,
  data: UpdateChunkGroupReqPayload,
  signal?: AbortSignal
) {
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

export async function updateGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: UpdateGroupByTrackingIDReqPayload,
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "put",
    {
      data,
      trackingId: data.tracking_id,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function addChunkToGroup(
  /** @hidden */
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { group_id: string },
  signal?: AbortSignal
) {
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

export async function removeChunkFromGroup(
  /** @hidden */
  this: TrieveSDK,
  data: RemoveChunkFromGroupReqPayload & { group_id: string },
  signal?: AbortSignal
) {
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

export async function getGroupsForChunks(
  /** @hidden */
  this: TrieveSDK,
  data: GetGroupsForChunksReqPayload,
  signal?: AbortSignal
) {
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

export async function getChunksGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunksInGroupByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
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

export async function getGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetGroupByTrackingIdData, "trDataset">,
  signal?: AbortSignal
) {
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

export async function addChunkToGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { trackingId: string },
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "post",
    {
      data,
      datasetId: this.datasetId,
      trackingId: data.trackingId,
    },
    signal
  );
}

export async function deleteGroupByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  data: DeleteGroupByTrackingIdData & { trackingId: string },
  signal?: AbortSignal
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "delete",
    {
      ...data,
      trackingId: data.trackingId,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunkGroupData, "trDataset">,
  signal?: AbortSignal
) {
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

export async function deleteGroup(
  /** @hidden */
  this: TrieveSDK,
  data: DeleteChunkGroupData,
  signal?: AbortSignal
) {
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

export async function getChunksInGroup(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetChunksInGroupData, "trDataset">,
  signal?: AbortSignal
) {
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

export async function getGroupsForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<Omit<GetGroupsForDatasetData, "datasetId">, "trDataset">,
  signal?: AbortSignal
) {
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
