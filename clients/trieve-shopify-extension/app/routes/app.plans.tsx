// plans.tsx
import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import { useLoaderData, useNavigate } from "@remix-run/react";
import {
  Page,
  DataTable,
  Card,
  Text,
  Button,
  Banner,
  Box,
  BlockStack,
  InlineStack,
  Divider,
} from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import { useEnvs } from "app/context/useEnvs";
import { useCallback, useState } from "react";

export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args.request, false);
  const trieve = sdkFromKey(key);
  if (!key.currentDatasetId) {
    throw redirect("/app/setup");
  }

  const availablePlans = await trieve.getStripePlans();
  const organization = await trieve.getOrganizationById(key.organizationId!);

  return { availablePlans, organization };
};

export default function PlansPage() {
  const navigate = useNavigate();
  const { availablePlans, organization } = useLoaderData<typeof loader>();
  const [upgrading, setUpgrading] = useState(false);
  const [processingPlanId, setProcessingPlanId] = useState<string | null>(
    organization.plan?.id || null,
  );
  const envs = useEnvs();

  const formatCurrency = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  });

  const formatNumber = new Intl.NumberFormat("en-US");

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 Bytes";
    const k = 1000;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const handleUpgrade = useCallback(
    async (planId: string) => {
      setProcessingPlanId(planId);
      setUpgrading(true);

      try {
        window.open(
          `${envs.TRIEVE_BASE_URL}/api/stripe/payment_link/${planId}/${organization?.organization.id}`,
        );
      } catch (error) {
        console.error("Failed to upgrade plan:", error);
        setUpgrading(false);
        setProcessingPlanId(null);
      }
    },
    [organization?.organization.id],
  );

  // Current plan details
  const currentPlan = organization?.plan;

  return (
    <Page
      title="Subscription Plans"
      backAction={{ content: "Dashboard", onAction: () => navigate("/app") }}
    >
      <BlockStack gap="500">
        <Banner title="Choose the right plan for your needs" tone="info">
          <p>
            Upgrade your plan to get more chunks, datasets, and messages. You'll
            only be charged for what you use.
          </p>
        </Banner>

        <Card>
          <DataTable
            columnContentTypes={[
              "text",
              "text",
              "text",
              "text",
              "text",
              "text",
              "text",
              "text",
            ]}
            headings={[
              "Plan",
              "Price",
              "Chunks",
              "Users",
              "Datasets",
              "Storage",
              "Messages",
              "Action",
            ]}
            rows={[
              [
                "Free",
                formatCurrency.format(0) + "/mo",
                formatNumber.format(1000),
                formatNumber.format(1),
                formatNumber.format(1),
                formatBytes(500000000),
                formatNumber.format(500),
                <Button
                  disabled={currentPlan?.name === "Free"}
                  variant={currentPlan?.name === "Free" ? "plain" : "primary"}
                >
                  {currentPlan?.name === "Free" ? "Current Plan" : "Downgrade"}
                </Button>,
              ],
              ...(availablePlans ?? []).map((plan) => {
                const isCurrentPlan = currentPlan?.id === plan.id;
                return [
                  plan.name,
                  formatCurrency.format(plan.amount / 100) + "/mo",
                  formatNumber.format(plan.chunk_count),
                  formatNumber.format(plan.user_count),
                  formatNumber.format(plan.dataset_count),
                  formatBytes(plan.file_storage),
                  formatNumber.format(plan.message_count),
                  <Button
                    disabled={isCurrentPlan || processingPlanId === plan.id}
                    variant={isCurrentPlan ? "plain" : "primary"}
                    loading={processingPlanId === plan.id}
                    onClick={() => handleUpgrade(plan.id)}
                  >
                    {isCurrentPlan ? "Current Plan" : "Upgrade"}
                  </Button>,
                ];
              }),
            ]}
          />
        </Card>

        <Card>
          <BlockStack gap="400">
            <Box paddingInline="400" paddingBlockStart="400">
              <Text variant="headingMd" as="h2">
                Enterprise
              </Text>
            </Box>
            <Divider />
            <Box paddingInline="400" paddingBlockEnd="400">
              <BlockStack gap="400">
                <Text as="p">
                  Need more? Contact us for a custom enterprise plan tailored to
                  your specific needs.
                </Text>
                <InlineStack gap="300">
                  <Button url="mailto:humans@trieve.ai">Contact Sales</Button>
                  <Button url="tel:+16282224090" variant="plain">
                    +1 628-222-4090
                  </Button>
                </InlineStack>
              </BlockStack>
            </Box>
          </BlockStack>
        </Card>
      </BlockStack>
    </Page>
  );
}
