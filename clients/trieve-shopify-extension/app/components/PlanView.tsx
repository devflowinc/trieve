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
  Badge,
} from "@shopify/polaris";
import { ProgressBar } from "./ProgressBar";
import { TrievePlan } from "trieve-ts-sdk";

export const PlanView = ({
  plan,
  planItems,
  setShowCancelModal,
  usagePercentage,
}: {
  plan: TrievePlan | null | undefined;
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
            <InlineStack gap="400" align="center" blockAlign="center">
              <Text variant="headingMd" as="h2">
                Plan Details
              </Text>
              <Badge>{plan?.name}</Badge>
            </InlineStack>
            <InlineStack gap="200" align="center" blockAlign="center">
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
            </InlineStack>
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
          <BlockStack>
            <span className="text-sm font-bold pb-1">Usage</span>
            <ProgressBar progress={usagePercentage} />
          </BlockStack>
          <DescriptionList items={planItems} />
        </Box>
      </BlockStack>
    </Card>
  );
};
