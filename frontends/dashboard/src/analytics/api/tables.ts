import {
  AnalyticsFilter,
  EventAnalyticsFilter,
  EventResponse,
  RAGAnalyticsFilter,
  RagQueryResponse,
  RAGSortBy,
  RecommendationsAnalyticsFilter,
  SearchQueryResponse,
  SearchSortBy,
  SortOrder,
} from "shared/types";
import { transformAnalyticsFilter } from "../utils/formatDate";
import { RecommendationsEventResponse } from "trieve-ts-sdk";

const apiHost = import.meta.env.VITE_API_HOST as string;

type SearchQueriesTablesParams = {
  filter?: AnalyticsFilter;
  page?: number;
  sortBy?: SearchSortBy;
  sortOrder?: SortOrder;
};

type RagQueriesTablesParams = {
  filter?: RAGAnalyticsFilter;
  page?: number;
  sortBy?: RAGSortBy;
  sortOrder?: SortOrder;
};

type RecommendationQueriesTablesParams = {
  filter?: RecommendationsAnalyticsFilter;
  page?: number;
  sortBy?: SearchSortBy;
  sortOrder?: SortOrder;
};

type EventTablesParams = {
  filter?: EventAnalyticsFilter;
  page?: number;
};

export const getSearchQueries = async (
  params: SearchQueriesTablesParams,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/search`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: params.filter
        ? transformAnalyticsFilter(params.filter)
        : undefined,
      page: params.page,
      sort_by: params.sortBy,
      sort_order: params.sortOrder,
      type: "search_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as SearchQueryResponse;
  return data.queries;
};

export const getRagQueries = async (
  params: RagQueriesTablesParams,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/rag`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: params.filter
        ? transformAnalyticsFilter(params.filter)
        : undefined,
      page: params.page,
      sort_by: params.sortBy,
      sort_order: params.sortOrder,
      type: "rag_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch rag queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as RagQueryResponse;
  return data.queries;
};

export const getEvents = async (
  params: EventTablesParams,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/events/all`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: params.filter
        ? transformAnalyticsFilter(params.filter)
        : undefined,
      page: params.page,
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch head queries: ${response.statusText}`);
  }

  const data = (await response.json()) as unknown as EventResponse;
  return data.events;
};

export const getRecommendationQueries = async (
  params: RecommendationQueriesTablesParams,
  datasetId: string,
) => {
  const response = await fetch(`${apiHost}/analytics/recommendations`, {
    credentials: "include",
    method: "POST",
    body: JSON.stringify({
      filter: params.filter
        ? transformAnalyticsFilter(params.filter)
        : undefined,
      page: params.page,
      sort_by: params.sortBy,
      sort_order: params.sortOrder,
      type: "recommendation_queries",
    }),
    headers: {
      "TR-Dataset": datasetId,
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to fetch recommendation queries: ${response.statusText}`,
    );
  }

  const data =
    (await response.json()) as unknown as RecommendationsEventResponse;
  return data.queries;
};
