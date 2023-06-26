import { readdirSync, readFileSync } from "fs";
import mime from "mime";
import fetch from "node-fetch";
import { getAuthCookie } from "./auth";

const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";

describe("File Upload Test", () => {
  test("User can upload the first docx file in the directory", async () => {
    const authCookie = await getAuthCookie();
    const directoryPath = "./demo-files/";

    // Read the first docx file in the directory
    const files = readdirSync(directoryPath);
    const docxFile = files.find((file) => file.endsWith(".docx"));
    const filePath = `${directoryPath}/${docxFile}`;

    // Read the file contents and convert to base64
    const fileContent = readFileSync(filePath, "base64url");
    const mimeType = mime.getType(filePath);

    // Prepare the request body
    const requestBody = {
      base64_docx_file: fileContent,
      file_name: docxFile,
      file_mime_type: mimeType,
      private: true,
    };

    // Make the POST request
    const response = await fetch(`${api_endpoint}/upload_file`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Cookie: authCookie,
      },
      credentials: "include",
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      console.error(await response.json());
    }

    // Assert the response or perform further checks if needed
    expect(response.status).toBe(200);
  }, 20000);
});
