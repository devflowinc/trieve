import {
  CreateTopicReqPayload,
  GetAllTopicsForOwnerIdData,
  UpdateTopicReqPayload,
} from "../../index";
import { TrieveSDK } from "../../sdk";

export async function createTopic(
  this: TrieveSDK,
  data: CreateTopicReqPayload
) {
  return await this.trieve.fetch("/api/topic", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function updateTopic(
  this: TrieveSDK,
  data: UpdateTopicReqPayload
) {
  return await this.trieve.fetch("/api/topic", "put", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getAllTopics(
  this: TrieveSDK,
  data: Omit<GetAllTopicsForOwnerIdData, "trDataset">
) {
  return await this.trieve.fetch("/api/topic/owner/{owner_id}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}
