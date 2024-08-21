import methods from "./functions/index";
import { TrieveFetchClient } from "./fetch-client";

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
}

type Methods = typeof methods;
Object.entries(methods).forEach(([name, method]) => {
  // @ts-expect-error string should be used to index in this case
  TrieveSDK.prototype[name] = method;
});

declare module "./sdk" {
  interface TrieveSDK extends Methods {}
}
