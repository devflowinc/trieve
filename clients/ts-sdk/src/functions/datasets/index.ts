/**
 * This includes all the functions you can use to communicate with our datasets endpoint
 *
 * @module Dataset Methods
 */

import { TrieveSDK } from "../../sdk";
import {
  CreateDatasetBatchReqPayload,
  CreateDatasetReqPayload,
  Dataset,
  DatasetAndUsage,
  DatasetQueueLengthsResponse,
  DatasetUsageCount,
  EventReturn,
  FileData,
  GetAllTagsReqPayload,
  GetAllTagsResponse,
  GetEventsData,
  GetPagefindIndexResponse,
  UpdateDatasetReqPayload,
} from "../../types.gen";

/**
 * Function that provides the ability to create a dataset. This function is used to create a new dataset in the organization.
 *
 * Example:
 * ```js
 * const dataset = await trieve.createDataset({
 *  dataset_name: "My Dataset",
 * });
 * ```
 */
export async function createDataset(
  /** @hidden */
  this: TrieveSDK,
  props: CreateDatasetReqPayload,
  signal?: AbortSignal
): Promise<Dataset> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to create a dataset");
  }

  return this.trieve.fetch(
    "/api/dataset",
    "post",
    {
      data: props,
      organizationId: this.organizationId,
    },
    signal
  ) as Promise<Dataset>;
}

/**
 * Function that provides the ability to update a dataset. This function is used to update an existing dataset in the organization by ID or Tracking ID.
 *
 * Example:
 * ```js
 * const dataset = await trieve.updateDataset({
 *   tracking_id: "123456",
 *   dataset_name: "change to this name",
 * });
 * ```
 */
export async function updateDataset(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateDatasetReqPayload,
  signal?: AbortSignal
): Promise<Dataset> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to update a dataset");
  }

  return this.trieve.fetch(
    "/api/dataset",
    "put",
    {
      data: props,
      organizationId: this.organizationId,
    },
    signal
  ) as Promise<Dataset>;
}

/**
 * Function that provides the ability to create datasets in batch. This function is used to create multiple datasets in the organization.
 *
 * Example:
 * ```js
 * const datasets = await trieve.batchCreateDatasets({
 *  datasets: [
 *    {
 *       dataset_name: "My Dataset 1",
 *    },
 *  ]});
 * ```
 */
export async function batchCreateDatasets(
  /** @hidden */
  this: TrieveSDK,
  props: CreateDatasetBatchReqPayload,
  signal?: AbortSignal
): Promise<Dataset[]> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to create a dataset");
  }

  return this.trieve.fetch(
    "/api/dataset/batch_create_datasets",
    "post",
    {
      data: props,
      organizationId: this.organizationId,
    },
    signal
  ) as Promise<Dataset[]>;
}

/**
 * Function that provides the ability to remove all data from a dataset. This function is used to clear all data from a dataset.
 *
 * Example:
 * ```js
 * await trieve.clearDataset("1111-2222-3333-4444");
 */
export async function clearDataset(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<void> {
  return this.trieve.fetch(
    "/api/dataset/clear/{dataset_id}",
    "put",
    {
      datasetId,
    },
    signal
  ) as Promise<void>;
}

export async function getDatasetEvents(
  /** @hidden */
  this: TrieveSDK,
  props: GetEventsData,
  datasetId: string,
  signal?: AbortSignal
): Promise<EventReturn> {
  return this.trieve.fetch(
    "/api/dataset/events",
    "post",
    {
      datasetId,
      data: props,
    },
    signal
  ) as Promise<EventReturn>;
}

export async function getDatasetFiles(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  page: number,
  signal?: AbortSignal
): Promise<FileData> {
  return this.trieve.fetch(
    "/api/dataset/files/{dataset_id}/{page}",
    "get",
    {
      datasetId,
      page,
    },
    signal
  ) as Promise<FileData>;
}

export async function getAllDatasetTags(
  /** @hidden */
  this: TrieveSDK,
  props: GetAllTagsReqPayload,
  datasetId: string,
  signal?: AbortSignal
): Promise<GetAllTagsResponse> {
  return this.trieve.fetch(
    "/api/dataset/get_all_tags",
    "post",
    {
      data: props,
      datasetId,
    },
    signal
  ) as Promise<GetAllTagsResponse>;
}

export async function getDatasetsFromOrganization(
  /** @hidden */
  this: TrieveSDK,
  organizationId: string,
  limit?: number,
  offset?: number,
  signal?: AbortSignal
): Promise<DatasetAndUsage[]> {
  return this.trieve.fetch(
    "/api/dataset/organization/{organization_id}",
    "get",
    {
      organizationId,
      limit,
      offset,
    },
    signal
  ) as Promise<DatasetAndUsage[]>;
}

export async function getDatasetByTrackingId(
  /** @hidden */
  this: TrieveSDK,
  trackingId: string,
  signal?: AbortSignal
): Promise<Dataset> {
  if (!this.organizationId) {
    throw new Error(
      "Organization ID is required to get a dataset by tracking ID"
    );
  }

  return this.trieve.fetch(
    "/api/dataset/tracking_id/{tracking_id}",
    "get",
    {
      organizationId: this.organizationId,
      trackingId,
    },
    signal
  ) as Promise<Dataset>;
}

export async function getDatasetUsageById(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<DatasetUsageCount> {
  return this.trieve.fetch(
    "/api/dataset/usage/{dataset_id}",
    "get",
    {
      datasetId,
    },
    signal
  ) as Promise<DatasetUsageCount>;
}

export async function getDatasetById(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<Dataset> {
  return this.trieve.fetch(
    "/api/dataset/{dataset_id}",
    "get",
    {
      datasetId,
    },
    signal
  ) as Promise<Dataset>;
}

export async function deleteDataset(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<void> {
  return this.trieve.fetch(
    "/api/dataset/{dataset_id}",
    "delete",
    {
      datasetId,
    },
    signal
  ) as Promise<void>;
}

export async function getPagefindUrl(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<GetPagefindIndexResponse> {
  return this.trieve.fetch(
    "/api/dataset/pagefind",
    "get",
    {
      datasetId,
    },
    signal
  ) as Promise<GetPagefindIndexResponse>;
}

export async function getDatasetQueueLengths(
  /** @hidden */
  this: TrieveSDK,
  datasetId: string,
  signal?: AbortSignal
): Promise<DatasetQueueLengthsResponse> {
  return this.trieve.fetch(
    "/api/dataset/get_dataset_queue_lengths",
    "get",
    {
      datasetId,
    },
    signal
  ) as Promise<DatasetQueueLengthsResponse>;
}
