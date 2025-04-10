import { useSubmit } from "@remix-run/react";
import {
  BlockStack,
  Box,
  Button,
  Card,
  DescriptionList,
  DescriptionListProps,
  InlineStack,
  Text,
  Banner,
} from "@shopify/polaris";
import { ProgressBar } from "./ProgressBar";

export const PlanView = ({
  planItems,
  setShowCancelModal,
  usagePercentage,
}: {
  planItems: DescriptionListProps["items"];
  setShowCancelModal: (show: boolean) => void;
  usagePercentage: number;
}) => {
  const submit = useSubmit();

  return (
    <Card>
      <BlockStack gap="400">
        <Box paddingInline="400" paddingBlockStart="400">
          <InlineStack align="space-between">
            <Text variant="headingMd" as="h2">
              Plan Details
            </Text>
            <div className="flex gap-2">
              <Button
                onClick={() => {
                  submit(
                    {
                      action: "modify",
                    },
                    {
                      method: "post",
                    },
                  );
                }}
              >
                Modify
              </Button>
              <Button
                onClick={() => {
                  setShowCancelModal(true);
                }}
              >
                Cancel
              </Button>
            </div>
          </InlineStack>
        </Box>

        <Box paddingInline="400" paddingBlockEnd="400">
          {usagePercentage >= 80 && usagePercentage < 90 && (
            <Box paddingBlockEnd="400">
              <Banner
                title={`You are at ${usagePercentage.toPrecision(5)}% of your usage limit.`}
                tone="warning"
              >
                <p>
                  Consider upgrading your plan to avoid potential disruptions.
                </p>
              </Banner>
            </Box>
          )}
          {usagePercentage >= 90 && (
            <Box paddingBlockEnd="400">
              <Banner
                title={`You have reached ${usagePercentage.toPrecision(5)}% of your usage limit.`}
                tone="critical"
              >
                <p>
                  Upgrade your plan immediately to avoid service disruption.
                </p>
              </Banner>
            </Box>
          )}
          <div className="w-full">
            <span className="text-sm font-bold pb-1">Usage</span>
            <ProgressBar progress={usagePercentage} />
          </div>
          <DescriptionList items={planItems} />
        </Box>
      </BlockStack>
    </Card>
  );
};
