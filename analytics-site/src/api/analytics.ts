import {
  AnalyticsParams,
  HeadQuery,
  LatencyDatapoint,
  RpsDatapoint,
  SearchQueryEvent,
} from "shared/types";
import { apiHost } from "../utils/apiHost";
import { transformParams } from "../utils/formatDate";

export const getLatency = async (
  filters: AnalyticsParams,
  datasetId: string,
): Promise<LatencyDatapoint[]> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/latency`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify(transformParams(filters)),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as LatencyDatapoint[];
  return data;
};

export const getRps = async (
  filters: AnalyticsParams,
  datasetId: string,
): Promise<RpsDatapoint[]> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/rps`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify(transformParams(filters)),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RpsDatapoint[];
  return data;
};

export const getHeadQueries = async (
  filters: AnalyticsParams,
  datasetId: string,
  page: number,
): Promise<HeadQuery[]> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/query/head`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({ filter: transformParams(filters), page: page }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as HeadQuery[];
  return data;
};

export const getLowConfidenceQueries = async (
  filters: AnalyticsParams,
  datasetId: string,
  page: number,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(
    `${apiHost}/analytics/${datasetId}/query/low_confidence`,
    {
      credentials: "include",
      method: "POST",
      body: JSON.stringify({ filter: transformParams(filters), page: page }),
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
  return data;
};
