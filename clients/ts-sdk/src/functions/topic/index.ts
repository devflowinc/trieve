/**
 * This includes all the functions you can use to communicate with our topics API
 *
 * @module Topic Methods
 */
import {
  CreateTopicReqPayload,
  DeleteTopicData2,
  GetAllTopicsForOwnerIdData,
  UpdateTopicReqPayload,
  CloneTopicReqPayload
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

/**
 * Create a new chat topic. Topics are attached to a owner_id’s and act as a coordinator for conversation message history of gen-AI chat sessions. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.createTopic({
  first_user_message: "hello",
  name: "Test",
  owner_id: "3c90c3cc-1d76-27198-8888-8dd25736052a",
});
 * ```
 */
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

/**
 * Clone a chat topic and all its messages to a new topic. Topics are attached to a owner_id’s and act as a coordinator for conversation message history of gen-AI chat sessions. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.cloneTopic({
  first_user_message: "hello",
  name: "Test",
  owner_id: "3c90c3cc-1d76-27198-8888-8dd25736052a",
});
 * ```
 */
export async function cloneTopic(
  /** @hidden */
  this: TrieveSDK,
  data: CloneTopicReqPayload,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/topic/clone",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}

/**
 * Update an existing chat topic. Currently, only the name of the topic can be updated. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.updateTopic({
  topic_id: "3c90c3cc-1d76-27198-8888-8dd25736052a",
  name: "NewName"
});
 * ```
 */
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

/**
 * Get all topics belonging to an arbitary owner_id. This is useful for managing message history and chat sessions. It is common to use a browser fingerprint or your user’s id as the owner_id. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.getAllTopics({
  ownerId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
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

/**
 * Delete an existing chat topic. When a topic is deleted, all associated chat messages are also deleted. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteTopic({
  topicId: "3c90c3cc-1d76-27198-8888-8dd25736052a"
});
 * ```
 */
export async function deleteTopic(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<DeleteTopicData2, "trDataset">,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/topic/{topic_id}",
    "delete",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal
  );
}
