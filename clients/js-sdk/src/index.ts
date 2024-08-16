import { Trieve } from "../../fetch-client/dist/index";
import {
  CountSearchMethod,
  SearchResponseBody,
} from "../../fetch-client/dist/types.gen";

export class TrieveSDK {
  trieve: Trieve;
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
  }

  async search({
    datasetId,
    search_type = "fulltext",
    query,
  }: {
    datasetId: string;
    search_type?: CountSearchMethod;
    query: string;
  }) {
    const searchResult = (await this.trieve.fetch("/api/chunk/search", "post", {
      xApiVersion: "V2",
      data: {
        search_type: search_type,
        query: query,
      },
      datasetId: datasetId,
    })) as SearchResponseBody;

    return searchResult;
  }
}
