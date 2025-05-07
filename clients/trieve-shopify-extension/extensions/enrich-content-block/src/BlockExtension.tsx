import { useEffect, useState } from "react";
import {
  reactExtension,
  BlockStack,
  AdminBlock,
  useApi,
  TextField,
  Button,
  InlineStack,
  Banner,
  TextArea,
} from "@shopify/ui-extensions-react/admin";
import { TrieveProvider } from "./TrieveProvider";
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
  const [content, setContent] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);

  const { extraContent, updateContent } =
    useChunkExtraContent(simplifiedProductId);

  useEffect(() => {
    if (extraContent) {
      setContent(extraContent);
    }
  }, [extraContent]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await updateContent(content);
      setShowSuccess(true);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <AdminBlock title="Enrich Content">
      <BlockStack gap="base">
        {showSuccess && (
          <Banner tone="success" onDismiss={() => setShowSuccess(false)}>
            Content saved successfully
          </Banner>
        )}
        <TextArea
          label="Product Content"
          value={content}
          onChange={setContent}
        />
        <InlineStack gap="base" inlineAlignment="end">
          <Button onPress={handleSave} disabled={isSaving}>
            {isSaving ? "Saving..." : "Save Content"}
          </Button>
        </InlineStack>
      </BlockStack>
    </AdminBlock>
  );
}
