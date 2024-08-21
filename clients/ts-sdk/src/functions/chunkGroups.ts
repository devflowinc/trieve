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
} from "../fetch-client";
import { TrieveSDK } from "../sdk";

export async function createChunkGroup(
  this: TrieveSDK,
  data: Omit<CreateChunkGroupReqPayloadEnum, "datasetId">
) {
  return this.trieve.fetch("/api/chunk_group", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function searchOverGroups(
  this: TrieveSDK,
  data: SearchOverGroupsReqPayload
) {
  return this.trieve.fetch("/api/chunk_group/group_oriented_search", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function searchInGroup(
  this: TrieveSDK,
  data: SearchWithinGroupReqPayload
) {
  return this.trieve.fetch("/api/chunk_group/search", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function recommendedGroups(
  this: TrieveSDK,
  data: RecommendGroupsReqPayload
) {
  return this.trieve.fetch("/api/chunk_group/recommend", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function updateGroup(
  this: TrieveSDK,
  data: UpdateChunkGroupReqPayload
) {
  return this.trieve.fetch("/api/chunk_group", "put", {
    data,
    datasetId: this.datasetId,
  });
}

export async function updateGroupByTrackingId(
  this: TrieveSDK,
  data: UpdateGroupByTrackingIDReqPayload
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "put",
    {
      data,
      trackingId: data.tracking_id,
      datasetId: this.datasetId,
    }
  );
}

export async function addChunkToGroup(
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { groupId: string }
) {
  return this.trieve.fetch("/api/chunk_group/chunk/{group_id}", "post", {
    data,
    groupId: data.groupId,
    datasetId: this.datasetId,
  });
}

export async function removeChunkFromGroup(
  this: TrieveSDK,
  data: RemoveChunkFromGroupReqPayload & { groupId: string }
) {
  return this.trieve.fetch("/api/chunk_group/chunk/{group_id}", "delete", {
    data,
    groupId: data.groupId,
    datasetId: this.datasetId,
  });
}

export async function getGroupsForChunks(
  this: TrieveSDK,
  data: GetGroupsForChunksReqPayload
) {
  return this.trieve.fetch("/api/chunk_group/chunks", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getChunksGroupByTrackingId(
  this: TrieveSDK,
  data: Omit<GetChunksInGroupByTrackingIdData, "trDataset">
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{group_tracking_id}/{page}",
    "get",
    {
      ...data,
      xApiVersion: data.xApiVersion || "V2",

      datasetId: this.datasetId,
    }
  );
}

export async function getGroupByTrackingId(
  this: TrieveSDK,
  data: Omit<GetGroupByTrackingIdData, "trDataset">
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    }
  );
}

export async function addChunkToGroupByTrackingId(
  this: TrieveSDK,
  data: AddChunkToGroupReqPayload & { trackingId: string }
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "post",
    {
      data,
      datasetId: this.datasetId,
      trackingId: data.trackingId,
    }
  );
}

export async function deleteGroupByTrackingId(
  this: TrieveSDK,
  data: DeleteGroupByTrackingIdData & { trackingId: string }
) {
  return this.trieve.fetch(
    "/api/chunk_group/tracking_id/{tracking_id}",
    "delete",
    {
      ...data,
      trackingId: data.trackingId,
      datasetId: this.datasetId,
    }
  );
}

export async function getGroup(
  this: TrieveSDK,
  data: Omit<GetChunkGroupData, "trDataset">
) {
  return this.trieve.fetch("/api/chunk_group/{group_id}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}

export async function deleteGroup(this: TrieveSDK, data: DeleteChunkGroupData) {
  return this.trieve.fetch("/api/chunk_group/{group_id}", "delete", {
    ...data,
    datasetId: this.datasetId,
  });
}

export async function getChunksInGroup(
  this: TrieveSDK,
  data: Omit<GetChunksInGroupData, "trDataset">
) {
  return this.trieve.fetch("/api/chunk_group/{group_id}/{page}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}

export async function getGroupsForDataset(
  this: TrieveSDK,
  data: Omit<Omit<GetGroupsForDatasetData, "datasetId">, "trDataset">
) {
  return this.trieve.fetch("/api/dataset/groups/{dataset_id}/{page}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}
