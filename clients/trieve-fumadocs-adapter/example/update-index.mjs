import { sync } from "trieve-fumadocs-adapter/search/sync";
import { TrieveSDK } from "trieve-ts-sdk";
import * as fs from "node:fs";

const content = fs.readFileSync(".next/server/app/static.json.body");

// now you can pass it to `sync`
/** @type {import('fumadocs-core/search/algolia').DocumentRecord[]} **/
const records = JSON.parse(content.toString());

const client = new TrieveSDK({
  apiKey: "tr-6HmidvkjyJ3tCbCCN6HGwpgAbkGEotvH",
  datasetId: "37af3aff-9d34-4003-ad3b-344a19e9f77e",
});

sync(client, records);
