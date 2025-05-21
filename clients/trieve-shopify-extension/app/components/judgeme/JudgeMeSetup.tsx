import {
  BlockStack,
  Box,
  Button,
  Card,
  InlineStack,
  SkeletonBodyText,
  Text,
} from "@shopify/polaris";
import { useMutation, useQuery } from "@tanstack/react-query";
import { type ApiType } from "app/api/root";
import { hc } from "hono/client";
import { JudgeMeSync } from "./JudgeMeSync";

export const JudgeMeSetup = () => {
  const client = hc<ApiType>("/");

  const {
    data: judgeMeInfo,
    status,
    error,
  } = useQuery({
    queryKey: ["judgeMeInfo"],
    queryFn: async () => {
      const repsonse = await client.api.judgeme.info.$get();
      if (!repsonse.ok) {
        throw new Error("Failed to get judge.me info", {
          cause: await repsonse.text(),
        });
      }
      const data = await repsonse.json();
      return data.judgeMeKey;
    },
  });

  const getRedirectUrlMutation = useMutation({
    mutationFn: async () => {
      const response = await client.api.judgeme.login.$get();
      if (!response.ok) {
        throw new Error("Failed to get redirect url", {
          cause: await response.text(),
        });
      }
      const data = await response.json();
      window.open(data.url, "_blank");
    },
  });

  // Set inner component
  let inner = null;
  if (status === "pending") {
    inner = <SkeletonBodyText></SkeletonBodyText>;
  } else if (status === "error") {
    inner = <div className="text-red-800">Error: {error.message}</div>;
  } else if (status === "success" && !judgeMeInfo) {
    inner = (
      <Button
        onClick={() => {
          getRedirectUrlMutation.mutate();
        }}
      >
        Setup Judge.me
      </Button>
    );
  } else if (status === "success" && judgeMeInfo) {
    inner = <JudgeMeSync />;
  }

  return (
    <Card>
      <BlockStack>
        <Box width="100%">
          <BlockStack>
            <Box width="100%">
              <InlineStack
                gap="1200"
                align="space-between"
                blockAlign="start"
                wrap={false}
              >
                <label>
                  <Text variant="headingMd" as="h6">
                    Judge.me Product Reviews
                  </Text>
                </label>
                <Box minWidth="fit-content">
                  <InlineStack align="end">{inner}</InlineStack>
                </Box>
              </InlineStack>
            </Box>
            <BlockStack gap="400">
              <Text variant="bodyMd" as="p" tone="subdued">
                Connect your Judge.me account to sync product reviews with
                Trieve.
              </Text>
            </BlockStack>
          </BlockStack>
        </Box>
      </BlockStack>
    </Card>
  );
};
