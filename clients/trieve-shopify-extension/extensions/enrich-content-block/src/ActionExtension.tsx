import { useEffect, useState } from "react";
import {
  reactExtension,
  useApi,
  AdminAction,
  BlockStack,
  Button,
  Text,
  AdminBlock,
} from "@shopify/ui-extensions-react/admin";

// The target used here must match the target used in the extension's toml file (./shopify.extension.toml)
const TARGET = "admin.product-details.block.render";

export default reactExtension(TARGET, () => <App />);

function App() {
  // The useApi hook provides access to several useful APIs like i18n, close, and data.
  const { i18n, data } = useApi(TARGET);
  console.log({ data });
  const [productTitle, setProductTitle] = useState("");
  // Use direct API calls to fetch data from Shopify.
  // See https://shopify.dev/docs/api/admin-graphql for more information about Shopify's GraphQL API
  useEffect(() => {
    (async function getProductInfo() {
      const getProductQuery = {
        query: `query Product($id: ID!) {
          product(id: $id) {
            title
          }
        }`,
        variables: { id: data.selected[0].id },
      };

      const res = await fetch("shopify:admin/api/graphql.json", {
        method: "POST",
        body: JSON.stringify(getProductQuery),
      });

      if (!res.ok) {
        console.error("Network error");
      }

      const productData = await res.json();
      setProductTitle(productData.data.product.title);
    })();
  }, [data.selected]);
  return (
    // The AdminAction component provides an API for setting the title and actions of the Action extension wrapper.
    <AdminBlock title="test">
      <BlockStack>
        {/* Set the translation values for each supported language in the locales directory */}
        <Text fontWeight="bold">
          {i18n.translate("welcome", { target: TARGET })}
        </Text>
        <Text>Current product: {productTitle}</Text>
      </BlockStack>
    </AdminBlock>
  );
}
