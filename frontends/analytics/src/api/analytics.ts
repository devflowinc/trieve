import {
  AnalyticsParams,
  HeadQuery,
  LatencyDatapoint,
  RagQueryEvent,
  RAGUsageResponse,
  RpsDatapoint,
  SearchQueryEvent,
  SearchTypeCount,
  LatencyGraphResponse,
  HeadQueryResponse,
  RagQueryResponse,
  SearchQueryResponse,
  QueryCountResponse,
  RPSGraphResponse,
  AnalyticsFilter,
  RecommendationAnalyticsFilter,
  RecommendationEventResponse,
} from "shared/types";
import { apiHost } from "../utils/apiHost";
import { transformAnalyticsFilter } from "../utils/formatDate";

export const getLatency = async (
  filters: AnalyticsFilter,
  granularity: AnalyticsParams["granularity"],
  datasetId: string,
): Promise<LatencyDatapoint[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      granularity: granularity,
      type: "latency_graph",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data: LatencyGraphResponse =
    (await response.json()) as unknown as LatencyGraphResponse;

  return data.latency_points;
};

export const getRps = async (
  filters: AnalyticsFilter,
  granularity: AnalyticsParams["granularity"],
  datasetId: string,
): Promise<RpsDatapoint[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      granularity,
      type: "rps_graph",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch trends bubbles: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RPSGraphResponse;
  return data.rps_points;
};

export const getHeadQueries = async (
  filters: AnalyticsFilter,
  datasetId: string,
  page: number,
): Promise<HeadQuery[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      page,
      type: "head_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as HeadQueryResponse;
  return data.queries;
};

export const getRAGQueries = async (
  datasetId: string,
  page: number,
): Promise<RagQueryEvent[]> => {
  const payload = {
    page,
  };

  const response = await fetch(`${apiHost}/analytics/rag`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      ...payload,
      type: "rag_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RagQueryResponse;
  return data.queries;
};

export const getRAGUsage = async (
  datasetId: string,
): Promise<RAGUsageResponse> => {
  const response = await fetch(`${apiHost}/analytics/rag`, {
    method: "POST",
    credentials: "include",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      type: "rag_usage",
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RAGUsageResponse;
  return data;
};

export const getLowConfidenceQueries = async (
  filters: AnalyticsFilter,
  datasetId: string,
  page: number,
  threshold?: number,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      page,
      threshold,
      type: "low_confidence_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch low confidence queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as SearchQueryResponse;
  return data.queries;
};

export const getNoResultQueries = async (
  filters: AnalyticsFilter,
  datasetId: string,
  page: number,
): Promise<SearchQueryEvent[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter(filters),
      page,
      type: "no_result_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch no result queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as SearchQueryResponse;
  return data.queries;
};

export const getQueryCounts = async (
  gt_date: Date,
  datasetId: string,
): Promise<SearchTypeCount[]> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: transformAnalyticsFilter({
        date_range: {
          gt: gt_date,
        },
      }),
      type: "count_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch no result queries: ${response.statusText}`,
    );
  }

  const data = (await response.json()) as unknown as QueryCountResponse;
  return data.total_queries;
};

export const getSearchQuery = async (
  datasetId: string,
  searchId: string,
): Promise<SearchQueryEvent> => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      search_id: searchId,
      type: "search_query",
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch search event: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryEvent;
  return data;
};

export const getLowConfidenceRecommendations = async ({
  filter,
  page,
  threshold,
  dataset,
}: {
  filter?: RecommendationAnalyticsFilter;
  page?: number;
  threshold?: number;
  dataset: string;
}) => {
  const response = await fetch(`${apiHost}/analytics/recommendations`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: filter ? transformAnalyticsFilter(filter) : undefined,
      page,
      threshold,
      type: "low_confidence_recommendations",
    }),
    headers: {
      "TR-Dataset": dataset,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch low confidence recommendations: ${response.statusText}`,
    );
  }

  const data =
    (await response.json()) as unknown as RecommendationEventResponse;
  return data.queries;
};
