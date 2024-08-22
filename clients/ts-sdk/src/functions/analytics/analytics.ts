import {
  ClusterAnalytics,
  CTRAnalytics,
  CTRDataRequestBody,
  RAGAnalytics,
  RecommendationAnalytics,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

export async function getCTRAnalytics(this: TrieveSDK, data: CTRAnalytics) {
  return await this.trieve.fetch("/api/analytics/ctr", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function sendCTRAnalytics(
  this: TrieveSDK,
  data: CTRDataRequestBody
) {
  return await this.trieve.fetch("/api/analytics/ctr", "put", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getRagAnalytics(this: TrieveSDK, data: RAGAnalytics) {
  return this.trieve.fetch("/api/analytics/rag", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getRecommendationAnalytics(
  this: TrieveSDK,
  data: RecommendationAnalytics
) {
  return this.trieve.fetch("/api/analytics/recommendations", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getSearchAnalytics(
  this: TrieveSDK,
  data: SearchAnalytics
) {
  return this.trieve.fetch("/api/analytics/search", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getClusterAnalytics(
  this: TrieveSDK,
  data: ClusterAnalytics
) {
  return this.trieve.fetch("/api/analytics/search/cluster", "post", {
    data,
    datasetId: this.datasetId,
  });
}
