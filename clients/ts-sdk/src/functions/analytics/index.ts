/**
 * This includes all the functions you can use to communicate with our analytics API
 *
 * @module Analytic Methods
 */

import {
  ClusterAnalytics,
  CTRAnalytics,
  CTRDataRequestBody,
  RAGAnalytics,
  RecommendationAnalytics,
  SearchAnalytics,
} from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

/**
 * Function that allows you to view the CTR analytics for a dataset:
 * Example:
 * ```js
 *const data = await trieve.getCTRAnalytics({
 *   type: "search_ctr_metrics",
 *});
 * ```
 */
export async function getCTRAnalytics(
  /** @hidden */
  /** @hidden */
  this: TrieveSDK,
  data: CTRAnalytics
) {
  return await this.trieve.fetch("/api/analytics/ctr", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function sendCTRAnalytics(
  /** @hidden */
  this: TrieveSDK,
  data: CTRDataRequestBody
) {
  return await this.trieve.fetch("/api/analytics/ctr", "put", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getRagAnalytics(
  /** @hidden */
  this: TrieveSDK,
  data: RAGAnalytics
) {
  return this.trieve.fetch("/api/analytics/rag", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getRecommendationAnalytics(
  /** @hidden */
  this: TrieveSDK,
  data: RecommendationAnalytics
) {
  return this.trieve.fetch("/api/analytics/recommendations", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getSearchAnalytics(
  /** @hidden */
  this: TrieveSDK,
  data: SearchAnalytics
) {
  return this.trieve.fetch("/api/analytics/search", "post", {
    data,
    datasetId: this.datasetId,
  });
}

export async function getClusterAnalytics(
  /** @hidden */
  this: TrieveSDK,
  data: ClusterAnalytics
) {
  return this.trieve.fetch("/api/analytics/search/cluster", "post", {
    data,
    datasetId: this.datasetId,
  });
}
