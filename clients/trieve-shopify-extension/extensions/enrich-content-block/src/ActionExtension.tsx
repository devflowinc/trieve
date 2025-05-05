import { useEffect, useState } from "react";
import {
  reactExtension,
  BlockStack,
  Text,
  AdminBlock,
  useApi,
} from "@shopify/ui-extensions-react/admin";
import { TrieveProvider, useTrieve } from "./TrieveProvider";

const TARGET = "admin.product-details.block.render";

export default reactExtension(TARGET, () => (
  <TrieveProvider>
    <App />
  </TrieveProvider>
));

function App() {
  const trieve = useTrieve();
  const { data } = useApi(TARGET);
  const productId = data.selected[0].id;

  return (
    <AdminBlock title="Enrich Content">
      <BlockStack>
        <Text fontWeight="bold">Hi</Text>
        <Text>Current product:{productId}</Text>
      </BlockStack>
    </AdminBlock>
  );
}
