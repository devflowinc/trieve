/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  SearchClusterResponse,
  SearchClusterTopics,
  SearchQueryEvent,
  SearchQueryResponse,
} from "shared/types";
import { apiHost } from "../utils/apiHost";

export const getTrendsBubbles = async (
  datasetId: string,
): Promise<SearchClusterTopics[]> => {
  const response = await fetch(`${apiHost}/analytics/search/clusters`, {
    credentials: "include",
    method: "POST",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      type: "cluster_topics",
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchClusterResponse;
  return data.clusters;
};

export const getQueriesForTopic = async (
  datasetId: string,
  clusterId: string,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(`${apiHost}/analytics/search/clusters`, {
    credentials: "include",
    method: "POST",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      type: "cluster_queries",
      cluster_id: clusterId,
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryResponse;
  return data.queries;
};
