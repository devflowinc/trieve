import { Button, Card, SkeletonBodyText, Text } from "@shopify/polaris";
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
      <Text variant="headingLg" as="h1">
        Connect Judge.me
      </Text>
      <Text variant="bodyMd" as="p">
        Connect your Judge.me reviews with your products in Trieve. Trieve will
        use your reviews to inform its responses when answering customer
        questions about products.
      </Text>

      <div className="h-3"></div>
      <div className="flex items-center gap-4">{inner}</div>
    </Card>
  );
};
