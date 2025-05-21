import { BlockStack, Box, Card, InlineGrid, Text } from "@shopify/polaris";
import { JudgeMeSetup } from "app/components/judgeme/JudgeMeSetup";

export function IntegrationsSettings() {
  return (
    <Box paddingInline="400">
      <BlockStack gap={{ xs: "800", sm: "400" }}>
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Product Reviews
              </Text>
              <Text as="p" variant="bodyMd">
                Enhance your product data with customer reviews, allowing Trieve
                to use review content for richer search results.
              </Text>
            </BlockStack>
          </Box>
          <BlockStack gap="400">
            <JudgeMeSetup />
          </BlockStack>
        </InlineGrid>
      </BlockStack>
    </Box>
  );
}
