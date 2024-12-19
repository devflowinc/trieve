import methods from "./functions/index";
import { TrieveFetchClient } from "./fetch-client";

export class TrieveSDK {
  trieve: TrieveFetchClient;
  datasetId?: string;
  organizationId?: string;
  constructor({
    apiKey,
    baseUrl = "https://api.trieve.ai",
    debug = false,
    datasetId,
    organizationId,
    omitCredentials,
  }: {
    apiKey: string;
    baseUrl?: string;
    debug?: boolean;
    datasetId?: string;
    organizationId?: string;
    omitCredentials?: boolean;
  }) {
    this.trieve = new TrieveFetchClient({
      apiKey,
      baseUrl,
      debug,
      organizationId,
      omitCredentials,
    });
    this.datasetId = datasetId;
    this.organizationId = organizationId;
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
