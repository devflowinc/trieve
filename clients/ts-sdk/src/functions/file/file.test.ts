import { beforeAll, describe, expect, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import {
  CreatePresignedUrlForCsvJsonResponseBody,
  FileData,
  FileDTO,
  GetFilesCursorResponseBody,
  UploadFileResponseBody,
} from "../../types.gen";
import { EXAMPLE_FILE_ID, TRIEVE } from "../../__tests__/constants";
import fs from "fs";
import { test } from "../../__tests__/utils";

const uploadMeFile = fs.readFileSync("./src/__tests__/uploadme.pdf");
const uploadMeFileEncoded = uploadMeFile.toString("base64");

const villageOfCatskillZoningRegulationsFile = fs.readFileSync(
  "./src/__tests__/Village_of_Catskill_Zoning_Regulations.pdf",
);
const villageOfCatskillZoningRegulationsFileEncoded =
  villageOfCatskillZoningRegulationsFile.toString("base64");

describe("File Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("uploadFile", async () => {
    const data = await trieve.uploadFile({
      base64_file: uploadMeFileEncoded,
      file_name: "uploadme.pdf",
      group_tracking_id: "file-upload-group",
    });
    expectTypeOf(data).toEqualTypeOf<UploadFileResponseBody>();
  });

  test("uploadFileWithChunkr", async () => {
    const data = await trieve.uploadFile({
      base64_file: villageOfCatskillZoningRegulationsFileEncoded,
      file_name: "Village_of_Catskill_Zoning_Regulations.pdf",
      group_tracking_id: "village-of-catskill-file-upload-group",
      chunkr_create_task_req_payload: {
        pipeline: "Chunkr",
      },
    });
    expectTypeOf(data).toEqualTypeOf<UploadFileResponseBody>();
  });

  test("createPresignedUrlForJsonl", async () => {
    const data = await trieve.createPresignedUrlForCsvJsonl({
      file_name: "flipkart.jsonl",
      group_tracking_id: "flipkart-file-upload-group",
      mappings: [
        {
          csv_jsonl_field: "product_url",
          chunk_req_payload_field: "link",
        },
        {
          csv_jsonl_field: "retail_price",
          chunk_req_payload_field: "num_value",
        },
        {
          csv_jsonl_field: "image",
          chunk_req_payload_field: "image_urls",
        },
        {
          csv_jsonl_field: "uniq_id",
          chunk_req_payload_field: "tracking_id",
        },
      ],
    });
    expectTypeOf(
      data,
    ).toEqualTypeOf<CreatePresignedUrlForCsvJsonResponseBody>();

    const presignedPutUrl = data.presigned_put_url;
    const fileResponse = await fetch(
      "https://trieve.b-cdn.net/csvjsonltesting/flipkart_com-ecommerce_sample.jsonl",
    );
    const blob = await fileResponse.blob();

    const uploadResponse = await fetch(presignedPutUrl, {
      method: "PUT",
      body: blob,
      headers: {
        "Content-Type": "text/jsonl",
      },
    });

    expect(uploadResponse.ok).toBeTruthy();
  });

  test("createPresignedUrlForCsv", async () => {
    const data = await trieve.createPresignedUrlForCsvJsonl({
      file_name: "uploadme.csv",
      group_tracking_id: "file-upload-group",
      mappings: [
        {
          csv_jsonl_field: "PassengerId",
          chunk_req_payload_field: "tracking_id",
        },
        {
          csv_jsonl_field: "Name",
          chunk_req_payload_field: "tag_set",
        },
      ],
    });
    expectTypeOf(
      data,
    ).toEqualTypeOf<CreatePresignedUrlForCsvJsonResponseBody>();

    const presignedPutUrl = data.presigned_put_url;
    const fileResponse = await fetch(
      "https://raw.githubusercontent.com/datasciencedojo/datasets/refs/heads/master/titanic.csv",
    );
    const blob = await fileResponse.blob();

    const uploadResponse = await fetch(presignedPutUrl, {
      method: "PUT",
      body: blob,
      headers: {
        "Content-Type": "text/csv",
      },
    });

    expect(uploadResponse.ok).toBeTruthy();
  });

  test("getFilesForDataset", async () => {
    const data = await trieve.getFilesForDataset({
      page: 1,
    });
    expectTypeOf(data).toEqualTypeOf<FileData>();
  });

  test("scrollFiles", async () => {
    const data = await trieve.scrollFiles({});
    expectTypeOf(data).toEqualTypeOf<GetFilesCursorResponseBody>();
  });

  test("getFile", async () => {
    const data = await trieve.getFile({
      fileId: EXAMPLE_FILE_ID,
    });
    expectTypeOf(data).toEqualTypeOf<FileDTO>();
  });
});
