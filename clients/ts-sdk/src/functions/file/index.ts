/**
 * This includes all the functions you can use to communicate with our file API
 *
 * @module File Methods
 */

import {
  $OpenApiTs,
  CreatePresignedUrlForCsvJsonlReqPayload,
  DeleteFileHandlerData,
  DeleteFileHandlerResponse,
  FileDTO,
  GetDatasetFilesAndGroupIdsHandlerData,
  GetFileHandlerData,
  GetFilesCursorHandlerData,
  UploadFileReqPayload,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

/**
 * Upload a file to S3 attached to the server. The file will be converted to HTML with tika and chunked algorithmically, images will be OCR’ed with tesseract. The resulting chunks will be indexed and searchable. Optionally, you can only upload the file and manually create chunks associated to the file after. See docs.trieve.ai and/or contact us for more details and tips. Auth’ed user must be an admin or owner of the dataset’s organization to upload a file.
 * 
 * Example:
 * ```js
 *const data = await trieve.uploadFile({
    base64_file: "base64_encoded_file",
    create_chunks: true,
    description: "This is an example file",
    file_mime_type: "application/pdf",
    file_name: "example.pdf",
    link: "https://example.com",
    metadata: {
      key1: "value1",
      key2: "value2",
    },
  });
 * ```
 */
export async function uploadFile(
  /** @hidden */
  this: TrieveSDK,
  data: UploadFileReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/file",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal,
  );
}

export async function createPresignedUrlForCsvJsonl(
  /** @hidden */
  this: TrieveSDK,
  data: CreatePresignedUrlForCsvJsonlReqPayload,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/file/csv_or_jsonl",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Scroll through the files along with their groups in a dataset. This is useful for paginating through files. The cursor is used to fetch the next page of files. The page size is used to specify how many files to fetch per page. The default page size is 10.
 * 
 * Example:
 * ```js
  *const data = await trieve.getFilesForDataset({
    cursor: "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8",
  });
 * ```
 */
export async function scrollFiles(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<Omit<GetFilesCursorHandlerData, "datasetId">, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/dataset/scroll_files",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get all files and their group ids which belong to a given dataset specified by the dataset_id parameter. 10 files and group ids are returned per page. This route may return the same file multiple times if the file is associated with multiple groups.
 * 
 * Example:
 * ```js
  *const data = await trieve.getFilesForDataset({
    page:1,
  });
 * ```
 */
export async function getFilesForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<
    Omit<GetDatasetFilesAndGroupIdsHandlerData, "datasetId">,
    "trDataset"
  >,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/dataset/files/{dataset_id}/{page}",
    "get",
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Get a signed s3 url corresponding to the file_id requested such that you can download the file.
 * 
 * Example:
 * ```js
 *const data = await trieve.getFile({
    fileId: "3c90c3cc-0d44-4b50-8888-8dd25736052a",
  });
 * ```
 */
export async function getFile(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<GetFileHandlerData, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return (await this.trieve.fetch(
    `/api/file/{file_id}${
      data.contentType
        ? `?content_type=${encodeURIComponent(data.contentType)}`
        : ""
    }` as unknown as keyof $OpenApiTs,
    "get" as unknown as never,
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal,
  )) as FileDTO;
}

/**
 * Delete a file from S3 attached to the server based on its id. This will disassociate chunks from the file, but only delete them all together if you specify delete_chunks to be true. Auth’ed user or api key must have an admin or owner role for the specified dataset’s organization.
 * 
 * Example:
 * ```js
 *const data = await trieve.deleteFile({
    fileId: "3c90c3cc-0d44-4b50-8888-8dd25736052a",
  });
 * ```
 */
export async function deleteFile(
  /** @hidden */
  this: TrieveSDK,
  data: Omit<DeleteFileHandlerData, "trDataset">,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return (await this.trieve.fetch(
    `/api/file/{file_id}${
      data.deleteChunks ? `?delete_chunks=${data.deleteChunks}` : ""
    }` as unknown as keyof $OpenApiTs,
    "delete" as unknown as never,
    {
      ...data,
      datasetId: this.datasetId,
    },
    signal,
  )) as DeleteFileHandlerResponse;
}
