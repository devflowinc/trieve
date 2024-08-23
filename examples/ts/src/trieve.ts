import { Trieve } from "trieve-fetch-client";

export const trieve = new Trieve({
  apiKey: "admin",
  baseUrl: "http://localhost:8090",
  debug: false,
});

export const EXAMPLE_DATASET_ID = "6e15c9ff-037b-4559-ad25-bbb17aaf51d2";
