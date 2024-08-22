/**
 * This includes all the functions you can use to communicate with our file API
 *
 * @module File Methods
 */

import {
  DeleteFileHandlerData,
  GetDatasetFilesHandlerData,
  GetFileHandlerData,
  UploadFileReqPayload,
} from "../../index";
import { TrieveSDK } from "../../sdk";

export async function uploadFile(
  /** @hidden */
  this: TrieveSDK,
  data: UploadFileReqPayload
) {
  return await this.trieve.fetch("/api/file", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getFilesForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<Omit<GetDatasetFilesHandlerData, "datasetId">, "trDataset">
) {
  return await this.trieve.fetch(
    "/api/dataset/files/{dataset_id}/{page}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    }
  );
}
export async function getFile(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetFileHandlerData, "trDataset">
) {
  return await this.trieve.fetch("/api/file/{file_id}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}

export async function deleteFile(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<DeleteFileHandlerData, "trDataset">
) {
  return await this.trieve.fetch("/api/file/{file_id}", "delete", {
    ...data,
    datasetId: this.datasetId,
  });
}
