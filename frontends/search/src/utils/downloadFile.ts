import { ChunkFile } from "../utils/apiTypes";

const apiHost = import.meta.env.VITE_API_HOST as string;

export const downloadFile = async (fileId: string, datasetId: string) => {
  const response = await fetch(`${apiHost}/file/${fileId}`, {
    credentials: "include",
    headers: {
"X-API-version": "2.0",
      "TR-Dataset": datasetId,
    },
  });

  if (!response.ok) {
    throw new Error("Failed to fetch dataset");
  }

  const data = (await response.json()) as ChunkFile;
  console.log(data);
  const possibleLink = data["s3_url"];
  if (possibleLink) {
    // Create a link element
    const link = document.createElement("a");
    // Set link's href to the file's URL
    link.href = possibleLink;
    // Set the download attribute to the file's name
    link.download = data["file_name"];

    link.target = "_blank";

    // Click the link
    link.click();
  }
};
