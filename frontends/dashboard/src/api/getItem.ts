import { ChunkMetadataStringTagSet } from "trieve-ts-sdk";

const apiHost = import.meta.env.VITE_API_HOST as string;

export const getChunkQuery = async (
  datasetId: string,
  chunkId: string,
): Promise<ChunkMetadataStringTagSet> => {
  const response = await fetch(`${apiHost}/chunk/${chunkId}`, {
    credentials: "include",
    headers: {
      "TR-Dataset": datasetId,
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch search event: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as ChunkMetadataStringTagSet;
  return data;
};
