import { readdir, stat, readFileSync } from "fs";
import { fileTypeFromBuffer } from "file-type";
import { join } from "path";
import fetch from "node-fetch";
import Keyv from "keyv";
import { getAuthCookie } from "../auth.js";

const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";
let authCookie = null;
getAuthCookie().then((cookie) => (authCookie = cookie));

const convertAndUpload = async (filePath) => {
  const dirFileBuf = readFileSync(filePath);
  // Check if the file read resulted in a buffer of length 0
  if (!dirFileBuf || dirFileBuf.length === 0) {
    console.error(`Error: ${filePath} is empty`);
    return;
  }

  let base64FileBuf = dirFileBuf.toString("base64");
  // Check if the base64 encoding resulted in a string
  if (
    !base64FileBuf ||
    base64FileBuf.length === 0 ||
    !(typeof base64FileBuf === "string")
  ) {
    console.error(`Error: ${filePath} could not be converted to base64`);
    return;
  }
  base64FileBuf = base64FileBuf
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");

  let fileMimeType = "";
  try {
    fileMimeType = (await fileTypeFromBuffer(dirFileBuf)).mime;
    if (!fileMimeType) {
      throw new Error("No file type data");
    }
  } catch (_err) {
    console.error(`Error: ${filePath} had no file type data`);
    return;
  }
  const fileName = filePath.split("/").pop().split(".")[0] + ".docx";

  const requestBody = {
    base64_docx_file: base64FileBuf,
    file_name: fileName,
    file_mime_type: fileMimeType,
    private: false,
  };

  await fetch(`${api_endpoint}/file`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Cookie: authCookie,
    },
    credentials: "include",
    body: JSON.stringify(requestBody),
  }).then((response) => {
    if (!response.ok) {
      console.error(
        `Error: ${response.status} ${response.statusText} for ${filePath}`
      );
      return;
    }
    console.log(`Uploaded ${filePath}`);
  });
};

const traverseDirectory = async (directoryPath) => {
  const keyvDb = new Keyv("redis://localhost:6380");

  return new Promise((resolve, reject) => {
    readdir(directoryPath, (err, files) => {
      if (err) {
        reject(err);
        return;
      }

      const promises = files.map((file) => {
        const filePath = join(directoryPath, file);

        return new Promise((resolve, reject) => {
          stat(filePath, async (err, stats) => {
            if (err) {
              reject(err);
              return;
            }

            if (stats.isDirectory()) {
              traverseDirectory(filePath).then(resolve).catch(reject);
            } else {
              // If file has already been uploaded, skip it
              const keyvRecord = await keyvDb.get(filePath);
              if (keyvRecord) {
                console.log(`Skipped ${filePath}, already uploaded`);
                resolve();
                return;
              }
              await keyvDb.set(filePath, true);
              convertAndUpload(filePath).then(resolve).catch(reject);
            }
          });
        });
      });

      Promise.all(promises).then(resolve).catch(reject);
    });
  });
};

// Usage: node script.js /path/to/directory

const directoryPath = process.argv[2];
if (!directoryPath) {
  console.error("Please provide a directory path.");
  process.exit(1);
}

const pollAuthCookie = () => {
  if (authCookie === null) {
    setTimeout(pollAuthCookie, 500);
  }
};
pollAuthCookie();

traverseDirectory(directoryPath)
  .then(() => {
    console.log("Traversal complete.");
    process.exit(0);
  })
  .catch((err) => {
    console.error("Traversal failed: ", err);
    process.exit(1);
  });
