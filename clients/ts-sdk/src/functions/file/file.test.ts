import { beforeAll, describe, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { File, FileDTO, UploadFileResult } from "../../types.gen";
import { EXAMPLE_FILE_ID, TRIEVE } from "../../__tests__/constants";
import fs from "fs";
import { test } from "../../__tests__/utils";

const file = fs.readFileSync("./src/__tests__/uploadme.pdf");

const fileEncoded = file
  .toString("base64")
  .replace(/\+/g, "-") // Convert '+' to '-'
  .replace(/\//g, "_") // Convert '/' to '_'
  .replace(/=+$/, ""); // Remove ending '='

describe("File Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("uploadFile", async () => {
    const data = await trieve.uploadFile({
      base64_file: fileEncoded,
      file_name: "uploadme.pdf",
      group_tracking_id: "file-upload-group",
    });
    expectTypeOf(data).toEqualTypeOf<UploadFileResult>();
  });

  test("getFilesForDataset", async () => {
    const data = await trieve.getFilesForDataset({
      page: 1,
    });
    expectTypeOf(data).toEqualTypeOf<File[]>();
  });

  test("getFile", async () => {
    const data = await trieve.getFile({
      fileId: EXAMPLE_FILE_ID,
    });
    expectTypeOf(data).toEqualTypeOf<FileDTO>();
  });
});
