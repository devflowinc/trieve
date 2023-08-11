import { readdir, stat, readFileSync } from "fs";
import { join } from "path";
import fetch from "node-fetch";
import Keyv from "keyv";
import { getAuthCookie } from "../auth.js";

const keyvDb = new Keyv(process.env.REDIS_URL || "redis://localhost:6380");
const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";
let authCookie = null;
const MAX_CONCURRENT_REQUESTS = process.env.MAX_CONCURRENT_REQUESTS || 1;
let activeRequests = 0;

const convertAndUpload = async (filePath, story_id) => {
  const dirFileBuf = readFileSync(filePath);
  const description = readFileSync(
    filePath.replace("-chapters", "-description")
  );
  // Check if the file read resulted in a buffer of length 0
  if (!dirFileBuf || dirFileBuf.length === 0) {
    console.error(`Error: ${filePath} is empty`);
    return;
  }

  let contentBase64FileBuf = dirFileBuf.toString("base64");
  let descriptionBase64FileBuf = description.toString("base64");
  // Check if the base64 encoding resulted in a string
  if (
    !contentBase64FileBuf ||
    contentBase64FileBuf.length === 0 ||
    !(typeof contentBase64FileBuf === "string")
  ) {
    console.error(`Error: ${filePath} could not be converted to base64`);
    return;
  }
  contentBase64FileBuf = contentBase64FileBuf
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");

  descriptionBase64FileBuf = descriptionBase64FileBuf
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");

  const fileName = filePath.split("/").pop();

  const requestBody = {
    base64_docx_file: contentBase64FileBuf,
    description: descriptionBase64FileBuf,
    file_name: fileName,
    file_mime_type: "text/html",
    oc_file_path: story_id,
    private: false,
  };

  // Acquire a slot in the semaphore
  while (activeRequests >= MAX_CONCURRENT_REQUESTS) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  activeRequests++;
  // If file has already been uploaded, skip it
  const keyvRecord = await keyvDb.get(story_id);
  if (keyvRecord) {
    console.log(
      `Skipped ${story_id}, already uploaded because of keyv ${keyvRecord}`
    );
    activeRequests--;
    return;
  }
  await keyvDb.set(story_id, true);

  await fetch(`${api_endpoint}/file`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Cookie: authCookie,
    },
    credentials: "include",
    body: JSON.stringify(requestBody),
  }).then(async (response) => {
    if (!response.ok) {
      console.error(
        `Error: ${response.status} ${response.statusText} for ${story_id}`
      );
      await keyvDb.set(`${story_id}_errored`, response.status);
      activeRequests--;
      return;
    }
    console.log(`Uploaded ${story_id}`);
    activeRequests--;
  });
};

const traverseDirectory = async (directoryPath) => {
  return new Promise((resolve, reject) => {
    readdir(directoryPath, (err, files) => {
      if (err) {
        reject(err);
        return;
      }

      const promises = files.map((file) => {
        const filePath = join(directoryPath, file);
        if (filePath.includes("description")) {
          return;
        }
        return new Promise((resolve, reject) => {
          stat(filePath, async (err, stats) => {
            if (err) {
              reject(err);
              return;
            }

            if (stats.isDirectory()) {
              traverseDirectory(filePath).then(resolve).catch(reject);
            } else {
              let truncatedFilePath = filePath.split("/").pop();
              let story_id = truncatedFilePath.split("-")[0];
              console.log(story_id);

              convertAndUpload(filePath, story_id)
                .then(resolve)
                .catch(() => {
                  resolve();
                });
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

getAuthCookie().then((cookie) => {
  authCookie = cookie;
  traverseDirectory(directoryPath)
    .then(() => {
      console.log("Traversal complete.");
      process.exit(0);
    })
    .catch((err) => {
      console.error("Traversal failed: ", err);
      process.exit(1);
    });
});
