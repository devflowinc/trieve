import {
  CreateTopicReqPayload,
  DeleteFileHandlerData,
  GetAllTopicsForOwnerIdData,
  GetDatasetFilesHandlerData,
  GetFileHandlerData,
  UpdateTopicReqPayload,
  UploadFileReqPayload,
} from "../../index";
import { TrieveSDK } from "../../sdk";

export async function uploadFile(this: TrieveSDK, data: UploadFileReqPayload) {
  return await this.trieve.fetch("/api/file", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getFilesForDataset(
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
  this: TrieveSDK,
  data: Omit<GetFileHandlerData, "trDataset">
) {
  return await this.trieve.fetch("/api/file/{file_id}", "get", {
    ...data,
    datasetId: this.datasetId,
  });
}

export async function deleteFile(
  this: TrieveSDK,
  data: Omit<DeleteFileHandlerData, "trDataset">
) {
  return await this.trieve.fetch("/api/file/{file_id}", "delete", {
    ...data,
    datasetId: this.datasetId,
  });
}
