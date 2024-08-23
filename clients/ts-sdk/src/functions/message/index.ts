/**
 * This includes all the functions you can use to communicate with our messages API
 *
 * @module Messages Methods
 */
import {
  CreateMessageReqPayload,
  EditMessageReqPayload,
  GetAllTopicMessagesData,
  RegenerateMessageReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

export async function createMessage(
  /** @hidden */
  this: TrieveSDK,
  data: CreateMessageReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/message",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function editMessage(
  /** @hidden */
  this: TrieveSDK,
  data: EditMessageReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/message",
    "put",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function regenerateMessage(
  /** @hidden */
  this: TrieveSDK,
  data: RegenerateMessageReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/message",
    "delete",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

export async function getAllMessagesForTopic(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetAllTopicMessagesData, "trDataset">,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/messages/{messages_topic_id}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}
