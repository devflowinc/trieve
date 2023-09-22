import { readdirSync, readFileSync, writeFileSync, unlinkSync } from "fs";
import mime from "mime";
import fetch from "node-fetch";
import { getAuthCookie } from "./auth";

const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";

describe("File Upload and Download Test", () => {
  let authCookie = null;

  test("User can upload the first docx file in the directory and then fetch it", async () => {
    authCookie = await getAuthCookie();
    const directoryPath = "./demo-files/";

    // Read the first docx file in the directory
    const files = readdirSync(directoryPath);
    const docxFile = files.find((file) => file.endsWith(".docx"));
    const filePath = `${directoryPath}/${docxFile}`;

    // Read the file contents and convert to base64
    const uploadFileContent = readFileSync(filePath, "base64url");
    const mimeType = mime.getType(filePath);

    // Prepare the request body
    const requestBody = {
      base64_docx_file: uploadFileContent,
      file_name: docxFile,
      file_mime_type: mimeType,
      private: false,
    };

    // Make the POST request
    const createFileResponse = await fetch(`${api_endpoint}/file`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: authCookie,
      },
      credentials: "include",
      body: JSON.stringify(requestBody),
    });

    expect(createFileResponse.ok).toBe(true);
    expect(createFileResponse.status).toBe(200);

    const createData = await createFileResponse.json();
    expect(createData).toHaveProperty("file_metadata");
    const file_metadata = createData.file_metadata;
    expect(file_metadata).toHaveProperty("id");
    const createdFileId = file_metadata.id;

    // Make the GET request
    const fetchFileResponse = await fetch(
      `${api_endpoint}/file/${createdFileId}`,
      {
        method: "GET",
        headers: {
          Cookie: authCookie,
        },
        credentials: "include",
      }
    );

    expect(fetchFileResponse.ok).toBe(true);
    expect(fetchFileResponse.status).toBe(200);

    const fetchData = await fetchFileResponse.json();
    expect(fetchData).toHaveProperty("id");
    expect(fetchData).toHaveProperty("file_name");
    expect(fetchData).toHaveProperty("mime_type");
    expect(fetchData).toHaveProperty("base64url_content");

    const file_name = fetchData.file_name;
    const base64url_content = fetchData.base64url_content;

    // Write the file to disk
    const fetchFileContent = Buffer.from(base64url_content, "base64url");
    const fetchFilePath = `./demo-files/${file_name} (downloaded).docx`;
    writeFileSync(fetchFilePath, fetchFileContent);

    // Check if the file was written to disk
    const filesAfterFetch = readdirSync(directoryPath);
    const downloadedFile = filesAfterFetch.find((file) =>
      file.endsWith("(downloaded).docx")
    );
    expect(downloadedFile).toBe(`${docxFile} (downloaded).docx`);

    // Check if the file contents are the same
    const downloadedFileContent = readFileSync(fetchFilePath, "base64url");
    expect(downloadedFileContent).toBe(uploadFileContent);

    // Delete the file from disk
    const deleteFilePath = `${directoryPath}/${downloadedFile}`;
    unlinkSync(deleteFilePath);

    // Check if the file was deleted from disk
    const filesAfterDelete = readdirSync(directoryPath);
    const deletedFile = filesAfterDelete.find((file) =>
      file.endsWith("(downloaded).docx")
    );
    expect(deletedFile).toBeUndefined();
  }, 40000);
});
