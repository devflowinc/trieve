import { Button, Card, Text } from "@shopify/polaris";
import { useMutation } from "@tanstack/react-query";
import { setMetafield } from "app/loaders";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { ONBOARD_STEP_META_FIELD } from "app/queries/onboarding";
import { onboardingSteps } from "app/utils/onboarding";

export const ResetSettings = () => {
  const adminApi = useClientAdminApi();

  const resetMetafieldsMutation = useMutation({
    onError: (e) => {
      console.error("Error clearing app metafields", e);
    },
    mutationFn: async () => {
      setMetafield(adminApi, ONBOARD_STEP_META_FIELD, onboardingSteps[0].id);
    },
  });

  return (
    <Card>
      <Text variant="headingLg" as="h1">
        Reset Onboarding
      </Text>
      <Text variant="bodyMd" as="p">
        This will reset your onboarding progress so you can view the steps
        again.
      </Text>
      <div className="h-3"></div>
      <div className="flex items-center gap-4">
        <Button
          onClick={() => {
            resetMetafieldsMutation.mutate();
          }}
          disabled={resetMetafieldsMutation.isPending}
          tone="critical"
        >
          Reset
        </Button>
        <div>
          {resetMetafieldsMutation.error && (
            <div className="text-red-500 opacity-100 start-hidden delay-75 duration-75 transition-opacity">
              {resetMetafieldsMutation.error.message}
            </div>
          )}
          {resetMetafieldsMutation.isSuccess && (
            <div className="opacity-80 start-hidden delay-75 duration-75 transition-opacity">
              Successfully reset app!
            </div>
          )}
        </div>
      </div>
    </Card>
  );
};
