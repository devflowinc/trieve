import { readdirSync, readFileSync } from 'fs';
import mime from 'mime';
import fetch from 'node-fetch';

describe('File Upload Test', () => {
  test('Uploads the first docx file in the directory', async () => {
    const directoryPath = './demo-files/';

    // Read the first docx file in the directory
    const files = readdirSync(directoryPath);
    const docxFile = files.find(file => file.endsWith('.docx'));
    const filePath = `${directoryPath}/${docxFile}`;

    // Read the file contents and convert to base64
    const fileContent = readFileSync(filePath, 'base64');
    const mimeType = mime.getType(filePath);

    // Prepare the request body
    const requestBody = {
      base64_docx_file: fileContent,
      file_name: docxFile,
      file_mime_type: mimeType,
      private: true,
    };

    const api_endpoint = process.env.API_ENDPOINT || 'http://localhost:8090/api';

    // Make the POST request
    const response = await fetch(`${api_endpoint}/upload_file`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(requestBody),
    });

    // Assert the response or perform further checks if needed
    expect(response.status).toBe(200);
  });
});
