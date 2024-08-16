import {
  CTRAnalytics,
  GetCtrAnalyticsData,
  RAGAnalytics,
  RecommendationAnalytics,
  SearchChunksReqPayload,
  SearchResponseBody,
  TrieveFetchClient,
} from "./fetch-client";

export class TrieveSDK {
  private trieve: TrieveFetchClient;
  private datasetId: string;
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

  async search(props: SearchChunksReqPayload) {
    const searchResults = (await this.trieve.fetch(
      "/api/chunk/search",
      "post",
      {
        xApiVersion: "V2",
        data: props,
        datasetId: this.datasetId,
      }
    )) as SearchResponseBody;

    return searchResults;
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
