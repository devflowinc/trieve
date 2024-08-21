import * as chunkMethods from "./functions/chunk";
import {
  CTRAnalytics,
  RAGAnalytics,
  RecommendationAnalytics,
  TrieveFetchClient,
} from "./fetch-client";

export class TrieveSDK {
  trieve: TrieveFetchClient;
  datasetId: string;
  constructor({
    apiKey,
    baseUrl = "https://api.trieve.ai",
    debug = false,
    datasetId,
  }: {
    apiKey: string;
    baseUrl?: string;
    debug?: boolean;
    datasetId: string;
  }) {
    this.trieve = new TrieveFetchClient({
      apiKey: apiKey,
      baseUrl: baseUrl,
      debug: debug,
    });
    this.datasetId = datasetId;
  }

  async getCTRAnalytics(props: CTRAnalytics) {
    return await this.trieve.fetch("/api/analytics/ctr", "post", {
      data: props,
      datasetId: this.datasetId,
    });
  }
  async getRagAnalytics(props: RAGAnalytics) {
    return this.trieve.fetch("/api/analytics/rag", "post", {
      data: props,
      datasetId: this.datasetId,
    });
  }
  async getRecommendationAnalytics(props: RecommendationAnalytics) {
    return this.trieve.fetch("/api/analytics/recommendations", "post", {
      data: props,
      datasetId: this.datasetId,
    });
  }
}
Object.entries(chunkMethods).forEach(([name, method]) => {
  // @ts-expect-error
  TrieveSDK.prototype[name] = method;
});

type Methods = typeof chunkMethods;
declare module "./sdk" {
  interface TrieveSDK extends Methods {}
}
