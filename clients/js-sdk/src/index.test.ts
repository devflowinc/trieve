// sum.test.js
import { expect, test } from "vitest";
import { TrieveSDK } from "./index";

test("adds 1 + 2 to equal 3", async () => {
  const trieve = new TrieveSDK({
    apiKey: "tr-mKHF9sstPHQHcCbh6Qk6Uw54hx7uwDGU",
  });

  const data = await trieve.search({
    query: "one",
    datasetId: "c04c43d9-382d-4815-810d-b776904a7390",
  });
  expect(data.chunks).toHaveLength(10);
});
