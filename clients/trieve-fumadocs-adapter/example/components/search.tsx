"use client";
import type { SharedProps } from "fumadocs-ui/components/dialog/search";
import SearchDialog from "trieve-fumadocs-adapter/components/dialog/search";
import { TrieveSDK } from "trieve-ts-sdk";

const trieveClient = new TrieveSDK({
  apiKey: "tr-SP2h9yW9yKmLWBvqL8JYNGNxjc7R48aQ",
  datasetId: "37af3aff-9d34-4003-ad3b-344a19e9f77e",
});

export default function CustomSearchDialog(props: SharedProps) {
  return <SearchDialog trieveClient={trieveClient} {...props} />;
}
