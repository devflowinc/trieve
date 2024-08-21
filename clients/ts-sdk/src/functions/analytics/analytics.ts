import {
  CTRAnalytics,
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
