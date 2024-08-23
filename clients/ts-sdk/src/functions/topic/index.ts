/**
 * This includes all the functions you can use to communicate with our topics API
 *
 * @module Topic Methods
 */
import {
  CreateTopicReqPayload,
  GetAllTopicsForOwnerIdData,
  UpdateTopicReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

export async function createTopic(
  /** @hidden */
  this: TrieveSDK,
  data: CreateTopicReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/topic",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function updateTopic(
  /** @hidden */
  this: TrieveSDK,
  data: UpdateTopicReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/topic",
    "put",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getAllTopics(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetAllTopicsForOwnerIdData, "trDataset">,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/topic/owner/{owner_id}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}
