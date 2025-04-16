import { Button, Card, Text } from "@shopify/polaris";
import { useMutation } from "@tanstack/react-query";

export const JudgeMeSetup = () => {
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

  return (
    <Card>
      <Text variant="headingLg" as="h1">
        Reset Widgets and Onboarding
      </Text>
      <Text variant="bodyMd" as="p">
        Clears all app metafields, onboarding data, and widget configuration.
      </Text>
      <div className="h-3"></div>
      <div className="flex items-center gap-4">
        <Button
          onClick={() => {
            getRedirectUrlMutation.mutate();
          }}
        >
          Setup Judge.me
        </Button>
      </div>
    </Card>
  );
};
