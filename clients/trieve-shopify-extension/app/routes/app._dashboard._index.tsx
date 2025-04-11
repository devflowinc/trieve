import { useTrieve } from "app/context/trieveContext";
import { useNavigate, useSubmit } from "@remix-run/react";
import {
  Card,
  Text,
  Badge,
  Button,
  SkeletonBodyText,
  DescriptionList,
  Box,
  BlockStack,
  InlineStack,
  Layout,
  Link,
  Modal,
} from "@shopify/polaris";
import {
  CalendarIcon,
  EnvelopeIcon,
  QuestionCircleIcon,
  RefreshIcon,
} from "@shopify/polaris-icons";
import { organizationUsageQuery, datasetUsageQuery } from "app/queries/usage";
import { useQuery } from "@tanstack/react-query";
import { Onboarding } from "app/components/onboarding/Onboarding";
import { Loader } from "app/loaders";
import { lastStepIdQuery } from "app/queries/onboarding";
import { createServerLoader } from "app/loaders/serverLoader";
import { createClientLoader } from "app/loaders/clientLoader";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { TopComponents } from "app/components/analytics/component/TopComponents";
import { TotalUniqueVisitors } from "app/components/analytics/component/TotalUniqueVisitors";
import { TopPages } from "app/components/analytics/component/TopPages";
import { useState } from "react";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { Granularity, StripePlan } from "trieve-ts-sdk";
import { ActionFunctionArgs } from "@remix-run/node";
import { authenticate } from "app/shopify.server";
import { PlanView } from "app/components/PlanView";

const currencyFormatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
});

const load: Loader = async ({ adminApiFetcher, queryClient }) => {
  await queryClient.ensureQueryData(lastStepIdQuery(adminApiFetcher));
  return;
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export const action = async ({ request }: ActionFunctionArgs) => {
  const { redirect, billing } = await authenticate.admin(request);
  const formData = await request.formData();
  const action = formData.get("action");
  if (action === "modify") {
    return redirect(process.env.SHOPIFY_PRICING_URL || "", {
      target: "_top",
    });
  } else if (action === "cancel") {
    const subscription = await billing.check();
    if (subscription.hasActivePayment) {
      await billing.cancel({
        subscriptionId: subscription.appSubscriptions[0].id,
      });
    }
    return redirect("/app");
  }
  return null;
};

export default function Dashboard() {
  const { organization, trieve, refetch: refetchTrieve } = useTrieve();
  const [filters, setFilters] = useState(defaultSearchAnalyticsFilter);
  const [granularity, setGranularity] = useState<Granularity>("day");
  const [showCancelModal, setShowCancelModal] = useState(false);
  const submit = useSubmit();

  const {
    data: datasetUsage,
    isLoading,
    dataUpdatedAt,
    refetch,
  } = useQuery(datasetUsageQuery(trieve));

  const { data: organizationUsage } = useQuery(organizationUsageQuery(trieve));

  const statsItems = [
    {
      term: "Products",
      description: isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : (
        <InlineStack align="space-between">
          {datasetUsage?.chunk_count.toLocaleString()}
          <Button
            icon={RefreshIcon}
            onClick={() => {
              refetch();
            }}
          ></Button>
        </InlineStack>
      ),
    },
    {
      term: "Last Synced",
      description: isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : dataUpdatedAt ? (
        new Date(dataUpdatedAt).toLocaleString()
      ) : (
        "Never"
      ),
    },
  ];

  let planItems = [];

  if (organization?.plan?.type === "flat") {
    planItems.push({
      term: "Message Usage",
      description: `${organizationUsage?.current_months_message_count?.toLocaleString() ?? 0} / ${((organization?.plan as StripePlan)?.messages_per_month ?? 1000).toLocaleString()}`,
    });
  }

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
      <Layout>
        <Layout.Section>
          <BlockStack gap="400">
            <Onboarding />
            <SearchFilterBar
              granularity={granularity}
              setGranularity={setGranularity}
              filters={filters}
              setFilters={setFilters}
              options={{ hideComponentName: true }}
            />
            <TopComponents filters={filters} />
            <SearchFilterBar
              granularity={granularity}
              setGranularity={setGranularity}
              filters={filters}
              setFilters={setFilters}
              options={{ hideDateRange: true }}
            />
            <Layout>
              <Layout.Section variant="oneHalf">
                <TotalUniqueVisitors
                  filters={filters}
                  granularity={granularity}
                />
              </Layout.Section>
              <Layout.Section variant="oneHalf">
                <TopPages filters={filters} />
              </Layout.Section>
            </Layout>
          </BlockStack>
        </Layout.Section>
        <Layout.Section variant="oneThird">
          <BlockStack gap="400">
            <Card>
              <BlockStack gap="400">
                <Text variant="headingLg" as="h1">
                  Get support
                </Text>
                <Text variant="bodyLg" as="p">
                  We would love to hear from you about anything at all. Email{" "}
                  <Link url="mailto:humans@trieve.ai" target="_blank">
                    humans@trieve.ai
                  </Link>{" "}
                  or call{" "}
                  <Link url="tel:+16282224090" target="_blank">
                    628-222-4090
                  </Link>{" "}
                  to quickly get in touch with a human on our team.
                </Text>
                <Text variant="bodyLg" as="p">
                  Or visit the{" "}
                  <Link
                    url="https://docs.trieve.ai/site-search/introduction"
                    target="_blank"
                  >
                    support center
                  </Link>{" "}
                  for answers to common questions, video tutorials,
                  documentation, and more.
                </Text>
                <InlineStack align="start" gap="300">
                  <Button
                    icon={QuestionCircleIcon}
                    size="large"
                    url="https://docs.trieve.ai/site-search/introduction"
                    target="_blank"
                  >
                    Support Center
                  </Button>
                  <Button
                    icon={EnvelopeIcon}
                    size="large"
                    url="mailto:humans@trieve.ai"
                    target="_blank"
                  >
                    Email Us
                  </Button>
                  <Button
                    icon={CalendarIcon}
                    size="large"
                    url="https://cal.com/team/trieve/chat"
                    target="_blank"
                  >
                    Book a Call
                  </Button>
                </InlineStack>
              </BlockStack>
            </Card>
            <Card>
              <BlockStack gap="400">
                <Box paddingInline="400" paddingBlockStart="400">
                  <InlineStack align="space-between">
                    <Text variant="headingMd" as="h2">
                      Sync Status
                    </Text>
                  </InlineStack>
                </Box>

                <Box paddingInline="400">
                  <DescriptionList gap="tight" items={statsItems} />
                </Box>

                <Box paddingInline="400" paddingBlockEnd="400">
                  <InlineStack align="end">
                    <Button
                      variant="primary"
                      onClick={() => {
                        fetch("/app/setup");
                      }}
                    >
                      Sync Index
                    </Button>
                  </InlineStack>
                </Box>
              </BlockStack>
            </Card>
            <PlanView
              plan={organization?.plan}
              planItems={planItems}
              setShowCancelModal={setShowCancelModal}
              usagePercentage={
                ((organizationUsage?.current_months_message_count ?? 0) /
                  ((organization?.plan as StripePlan)?.messages_per_month ??
                    1000)) *
                100
              }
            />
          </BlockStack>
        </Layout.Section>
      </Layout>
    </>
  );
}
