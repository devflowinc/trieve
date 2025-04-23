import { useSubmit } from "@remix-run/react";
import {
  BlockStack,
  Box,
  Button,
  Card,
  InlineStack,
  Text,
  Banner,
  Badge,
  Modal,
} from "@shopify/polaris";
import { ProgressBar } from "./ProgressBar";
import { StripePlan } from "trieve-ts-sdk";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { organizationUsageQuery } from "app/queries/usage";
import { useTrieve } from "app/context/trieveContext";

export const PlanView = () => {
  const { organization, trieve, refetch: refetchTrieve } = useTrieve();
  const [showCancelModal, setShowCancelModal] = useState(false);
  const submit = useSubmit();

  const { data: organizationUsage } = useQuery(organizationUsageQuery(trieve));

  let planItems = [];

  if (organization?.plan?.type === "flat") {
    planItems.push({
      term: "Message Usage",
      description: `${organizationUsage?.current_months_message_count?.toLocaleString() ?? 0} / ${((organization?.plan as StripePlan)?.messages_per_month ?? 1000).toLocaleString()}`,
    });
  }

  const usagePercentage =
    ((organizationUsage?.current_months_message_count ?? 0) /
      ((organization?.plan as StripePlan)?.messages_per_month ?? 1000)) *
    100;

  return (
    <>
      <Modal
        open={showCancelModal}
        onClose={() => {
          setShowCancelModal(false);
        }}
        title="Cancel Subscription"
      >
        <div className="flex flex-col gap-4 p-4">
          <Text as="p">Do you want to cancel your subscription?</Text>
          <Button
            onClick={() => {
              submit(
                {
                  action: "cancel",
                },
                {
                  method: "post",
                },
              );
              setShowCancelModal(false);
              setTimeout(() => {
                refetchTrieve();
              }, 5000);
            }}
          >
            Cancel Subscription
          </Button>
        </div>
      </Modal>
      <Card>
        <BlockStack>
          <div className="pb-4">
            <InlineStack align="space-between">
              <Text variant="headingMd" as="h2">
                Plan Details
              </Text>
              <Badge>{organization.plan?.name}</Badge>
            </InlineStack>
          </div>

          <Box>
            {usagePercentage >= 80 && usagePercentage < 90 && (
              <Box>
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
              <Box>
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
            {planItems.map((item) => (
              <div className="py-2">
                <div className="text-sm font-bold pb-1">{item.term}</div>
                <div>{item.description}</div>
              </div>
            ))}
          </Box>
        </BlockStack>
        <div className="h-2"></div>
        <InlineStack gap="200" align="end" blockAlign="center">
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
            Modify Plan
          </Button>
          <Button
            onClick={() => {
              setShowCancelModal(true);
            }}
          >
            Cancel Plan
          </Button>
        </InlineStack>
      </Card>
    </>
  );
};
