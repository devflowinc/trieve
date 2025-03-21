import { useTrieve } from "app/context/trieveContext";
import { useNavigate } from "@remix-run/react";
import {
  Card,
  Text,
  Badge,
  Banner,
  Button,
  SkeletonBodyText,
  DescriptionList,
  Box,
  BlockStack,
  InlineStack,
  Grid,
} from "@shopify/polaris";
import { RefreshIcon } from "@shopify/polaris-icons";
import { usageQuery } from "app/queries/usage";
import { useQuery } from "@tanstack/react-query";
import { Onboarding } from "app/components/Onboarding";
import { Loader } from "app/loaders";
import { lastStepIdQuery } from "app/queries/onboarding";
import { createServerLoader } from "app/loaders/serverLoader";
import { createClientLoader } from "app/loaders/clientLoader";

const load: Loader = async ({ adminApiFetcher, queryClient }) => {
  await queryClient.ensureQueryData(lastStepIdQuery(adminApiFetcher));
  return;
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export default function Dashboard() {
  const { dataset, organization, trieve } = useTrieve();

  const {
    data: usage,
    isLoading,
    dataUpdatedAt,
    refetch,
  } = useQuery(usageQuery(trieve));

  const planType = organization?.plan?.name || "Free";

  const statsItems = [
    {
      term: "Chunks",
      description: isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : (
        <InlineStack align="space-between">
          {usage?.chunk_count.toLocaleString()}{" "}
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
      term: "Last Updated",
      description: isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : dataUpdatedAt ? (
        new Date(dataUpdatedAt).toLocaleString()
      ) : (
        "Never"
      ),
    },
  ];

  const planItems = [
    {
      term: "Plan",
      description: planType,
    },
    {
      term: "Chunk Limit",
      description: organization?.plan?.chunk_count?.toLocaleString() || "N/A",
    },
    {
      term: "Dataset Limit",
      description: organization?.plan?.dataset_count?.toLocaleString() || "N/A",
    },
    {
      term: "Message Limit",
      description: organization?.plan?.message_count || "N/A",
    },
  ];

  const navigate = useNavigate();

  return (
    <BlockStack gap="400">
      <Onboarding />
      <Grid>
        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <Card>
            <BlockStack gap="400">
              <Box paddingInline="400" paddingBlockStart="400">
                <InlineStack align="space-between">
                  <Text variant="headingMd" as="h2">
                    Dataset Overview
                  </Text>
                  <Badge>{planType + " Plan"}</Badge>
                </InlineStack>
              </Box>

              <Box paddingInline="400">
                <DescriptionList items={statsItems} />
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
        </Grid.Cell>

        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <Card>
            <BlockStack gap="400">
              <Box paddingInline="400" paddingBlockStart="400">
                <InlineStack align="space-between">
                  <Text variant="headingMd" as="h2">
                    Plan Details
                  </Text>
                  <Button onClick={() => navigate("/app/plans")}>
                    Upgrade Plan
                  </Button>
                </InlineStack>
              </Box>

              <Box paddingInline="400">
                <DescriptionList items={planItems} />
              </Box>
            </BlockStack>
          </Card>
        </Grid.Cell>
      </Grid>
    </BlockStack>
  );
}
