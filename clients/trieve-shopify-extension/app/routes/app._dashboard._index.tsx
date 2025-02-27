import { useTrieve } from "app/context/trieveContext";
import { useNavigate } from "@remix-run/react";
import { useState, useEffect } from "react";
import {
  Card,
  Text,
  Badge,
  Banner,
  Button,
  Icon,
  SkeletonBodyText,
  DescriptionList,
  Box,
  BlockStack,
  InlineStack,
  Divider,
  Grid,
  InlineGrid,
} from "@shopify/polaris";
import { RefreshIcon } from "@shopify/polaris-icons";

export default function Dashboard() {
  const { dataset, organization, trieve } = useTrieve();
  const [stats, setStats] = useState({
    chunks: 0,
    lastUpdated: null as string | null,
    isLoading: true,
  });

  const fetchStats = async () => {
    try {
      const stats = await trieve.getDatasetUsageById(dataset.id);
      setStats({
        chunks: stats.chunk_count,
        lastUpdated: new Date().toISOString(),
        isLoading: false,
      });
    } catch (error) {
      console.error("Failed to fetch dataset stats:", error);
      setStats((prev) => ({ ...prev, isLoading: false }));
    }
  };

  useEffect(() => {
    if (dataset?.id) {
      fetchStats();
    }
  }, [dataset?.id]);

  const planType = organization?.plan?.name || "Free";

  const statsItems = [
    {
      term: "Chunks",
      description: stats.isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : (
        <InlineStack align="space-between">
          {stats.chunks.toLocaleString()}{" "}
          <Button
            icon={RefreshIcon}
            onClick={() => {
              fetchStats();
            }}
          ></Button>
        </InlineStack>
      ),
    },
    {
      term: "Last Updated",
      description: stats.isLoading ? (
        <SkeletonBodyText lines={1} />
      ) : stats.lastUpdated ? (
        new Date(stats.lastUpdated).toLocaleString()
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
      <Banner
        title={`Welcome to ${dataset?.name || "your dataset"}`}
        tone="info"
      >
        <p>
          This is your dataset dashboard where you can manage and search through
          your data.
        </p>
      </Banner>

      <Grid>
        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <Card>
            <BlockStack gap="400">
              <Box paddingInline="400" paddingBlockStart="400">
                <InlineStack align="space-between">
                  <Text variant="headingMd" as="h2">
                    Dataset Overview
                  </Text>
                  <Badge>{planType} Plan</Badge>
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
