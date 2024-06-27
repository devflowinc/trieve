import { SearchClusterTopics, SearchQueryEvent } from "shared/types";
import { apiHost } from "../utils/apiHost";

export const getTrendsBubbles = async (
  datasetId: string,
): Promise<SearchClusterTopics[]> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/topics`, {
    credentials: "include",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchClusterTopics[];
  return data;
};

export const getQueriesForTopic = async (
  datasetId: string,
  clusterId: string,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(
    `${apiHost}/analytics/${datasetId}/${clusterId}/1`,
    {
      credentials: "include",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryEvent[];
  console.log("GOT DATA", data);
  return data;
};
