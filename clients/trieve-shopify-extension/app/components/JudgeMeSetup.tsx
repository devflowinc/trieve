import { Button, Card, SkeletonBodyText, Text } from "@shopify/polaris";
import { useMutation, useQuery } from "@tanstack/react-query";
import { type JudgeMeKeyInfo } from "app/routes/api.judgeme.info";

export const JudgeMeSetup = () => {
  const {
    data: judgeMeInfo,
    status,
    error,
  } = useQuery({
    queryKey: ["judgeMeInfo"],
    queryFn: async () => {
      const repsonse = await fetch("/api/judgeme/info");
      if (!repsonse.ok) {
        throw new Error("Failed to get judge.me info", {
          cause: await repsonse.text(),
        });
      }
      const data = (await repsonse.json()) as JudgeMeKeyInfo;
      return data.judgeMeKey;
    },
  });

  const getRedirectUrlMutation = useMutation({
    mutationFn: async () => {
      const response = await fetch("/auth/judgeme/login");
      if (!response.ok) {
        throw new Error("Failed to get redirect url", {
          cause: await response.text(),
        });
      }
      const data = (await response.json()) as { url: string };
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
    inner = (
      <div className="text-green-800">Successfully connected to Judge.me</div>
    );
  }

  return (
    <Card>
      <Text variant="headingLg" as="h1">
        Connect Judge.me
      </Text>
      <Text variant="bodyMd" as="p">
        Connect your Judge.me reviews with your products in Trieve.
      </Text>

      <div className="h-3"></div>
      <div className="flex items-center gap-4">{inner}</div>
    </Card>
  );
};
