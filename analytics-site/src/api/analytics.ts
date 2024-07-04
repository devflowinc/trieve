/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  AnalyticsParams,
  HeadQuery,
  LatencyDatapoint,
  RagQueryEvent,
  RAGUsageResponse,
  RpsDatapoint,
  SearchQueryEvent,
} from "shared/types";
import { apiHost } from "../utils/apiHost";
import { transformAnalyticsParams } from "../utils/formatDate";

export const getLatency = async (
  filters: AnalyticsParams,
  datasetId: string,
): Promise<LatencyDatapoint[]> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/latency`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify(transformAnalyticsParams(filters)),
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
    body: JSON.stringify(transformAnalyticsParams(filters)),
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
    body: JSON.stringify(transformAnalyticsParams(filters, page)),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as HeadQuery[];
  return data;
};

export const getRAGQueries = async (
  datasetId: string,
  page: number,
): Promise<RagQueryEvent[]> => {
  const payload = {
    page,
  };

  const response = await fetch(`${apiHost}/analytics/${datasetId}/rag`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify(payload),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RagQueryEvent[];
  return data;
};

export const getRAGUsage = async (
  datasetId: string,
): Promise<RAGUsageResponse> => {
  const response = await fetch(`${apiHost}/analytics/${datasetId}/rag/usage`, {
    credentials: "include",
    method: "GET",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RAGUsageResponse;
  return data;
};

export const getLowConfidenceQueries = async (
  filters: AnalyticsParams,
  datasetId: string,
  page: number,
  threshold?: number,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(
    `${apiHost}/analytics/${datasetId}/query/low_confidence`,
    {
      credentials: "include",
      method: "POST",
      body: JSON.stringify({
        ...transformAnalyticsParams(filters, page),
        threshold,
      }),
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(
      `Failed to fetch low confidence queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as SearchQueryEvent[];
  return data;
};

export const getSearchQuery = async (
  datasetId: string,
  searchId: string,
): Promise<SearchQueryEvent> => {
  const response = await fetch(
    `${apiHost}/analytics/${datasetId}/query/${searchId}`,
    {
      credentials: "include",
      method: "GET",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch search event: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryEvent;
  return data;
};
