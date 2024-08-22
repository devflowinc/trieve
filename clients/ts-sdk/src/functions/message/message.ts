import {
  CreateMessageReqPayload,
  EditMessageReqPayload,
  GetAllTopicMessagesData,
  RegenerateMessageReqPayload,
} from "../../index";
import { TrieveSDK } from "../../sdk";

export async function createMessage(
  this: TrieveSDK,
  data: CreateMessageReqPayload
) {
  return await this.trieve.fetch("/api/message", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function updateMessage(
  this: TrieveSDK,
  data: EditMessageReqPayload
) {
  return await this.trieve.fetch("/api/message", "put", {
    data,
    datasetId: this.datasetId,
  });
}
export async function regenerateMessage(
  this: TrieveSDK,
  data: RegenerateMessageReqPayload
) {
  return await this.trieve.fetch("/api/message", "delete", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getAllMessagesForTopic(
  this: TrieveSDK,
  data: Omit<GetAllTopicMessagesData, "trDataset">
) {
  return await this.trieve.fetch("/api/messages/{messages_topic_id}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}
