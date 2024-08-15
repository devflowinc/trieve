import { Trieve } from "trieve-fetch-client";

export const trieve = new Trieve({
  apiKey: "admin",
  baseUrl: "http://localhost:8090",
  debug: false,
});

export const EXAMPLE_DATASET_ID = "9f2600a4-9979-4a54-a0e5-d4da4bb0eaec";
