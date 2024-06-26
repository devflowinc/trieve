import { SearchClusterTopics } from "shared/types";
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
