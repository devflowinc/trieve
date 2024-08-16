import { Trieve } from "../../fetch-client/dist/index";
import {
  CountSearchMethod,
  SearchResponseBody,
} from "../../fetch-client/dist/types.gen";

export class TrieveSDK {
  trieve: Trieve;
  datasetId: string | null;
  constructor({
    apiKey,
    baseUrl = "https://api.trieve.ai",
  }: {
    apiKey: string;
    baseUrl?: string;
  }) {
    this.trieve = new Trieve({
      apiKey: apiKey,
      baseUrl: baseUrl,
      debug: false,
    });
    this.datasetId = null;
  }

  dataset(datasetId: string) {
    this.datasetId = datasetId;

    return this;
  }

  async search({
    search_type = "fulltext",
    query,
  }: {
    search_type?: CountSearchMethod;
    query: string;
  }) {
    if (!this.datasetId) {
      console.log("No dataset passed");
      return;
    }
    const searchResults = (await this.trieve.fetch(
      "/api/chunk/search",
      "post",
      {
        xApiVersion: "V2",
        data: {
          search_type: search_type,
          query: query,
        },
        datasetId: this.datasetId,
      }
    )) as SearchResponseBody;

    return searchResults;
  }
}
