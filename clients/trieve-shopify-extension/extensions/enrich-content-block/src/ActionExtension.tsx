import { useEffect, useState } from "react";
import {
  reactExtension,
  BlockStack,
  Text,
  AdminBlock,
  useApi,
} from "@shopify/ui-extensions-react/admin";
import { TrieveProvider, useTrieve } from "./TrieveProvider";
import { useChunkExtraContent } from "./useChunkExtraContent";

const TARGET = "admin.product-details.block.render";

function extractShopifyProductId(gid: string): string | undefined {
  const parts = gid.substring(6).split("/"); // Remove "gid://" and then split
  if (parts.length === 3 && parts[0] === "shopify" && parts[1] === "Product") {
    return parts[2];
  }
  return undefined;
}

export default reactExtension(TARGET, () => (
  <TrieveProvider>
    <App />
  </TrieveProvider>
));

function App() {
  const { data } = useApi(TARGET);
  const productId = data.selected[0].id;
  const simplifiedProductId = extractShopifyProductId(productId);

  const { extraContent, loading, updateContent } =
    useChunkExtraContent(simplifiedProductId);

  return (
    <AdminBlock title="Enrich Content">
      <BlockStack>
        <Text fontWeight="bold">Hi</Text>
        <Text>Current product:{simplifiedProductId}</Text>
      </BlockStack>
    </AdminBlock>
  );
}
