import {
  Box,
  Card,
  Frame,
  Layout,
  Page,
  Text,
  BlockStack,
  InlineGrid,
  Tooltip,
  DescriptionList,
} from "@shopify/polaris";
import { useLoaderData } from "@remix-run/react";
import { loader } from "app/routes/app._dashboard.experimentview.$experimentId";
import { ExperimentAnalyticsChart } from "./analytics/ExperimentAnalyticsChart";

export function ExperimentView() {
  const {
    experiment,
    userCountsResult: variantStats,
    conversionStats,
  } = useLoaderData<typeof loader>();

  const controlVariant = variantStats.find(
    (v) => v.treatment_name === experiment.control_name,
  );
  const treatmentVariant = variantStats.find(
    (v) => v.treatment_name === experiment.t1_name,
  );

  const controlConversionStat = conversionStats.find(
    (cs) => cs.treatment_name === experiment.control_name,
  );
  const treatmentConversionStat = conversionStats.find(
    (cs) => cs.treatment_name === experiment.t1_name,
  );

  return (
    <Frame>
      <Page
        title={`Experiment Report: ${experiment.name}`}
        backAction={{
          content: "Back to Experiments",
          url: "/app/experiments",
        }}
      >
        <Layout>
          <Layout.Section>
            <BlockStack gap="500">
              <Card>
                <Box padding="400">
                  <BlockStack gap="300">
                    <Text variant="headingMd" as="h2">
                      Experiment Configuration
                    </Text>
                    <DescriptionList
                      items={[
                        {
                          term: "Control",
                          description: `${experiment.control_name} (${(experiment.control_split * 100).toFixed(0)}%)`,
                        },
                        {
                          term: "Treatment",
                          description: `${experiment.t1_name} (${(experiment.t1_split * 100).toFixed(0)}%)`,
                        },
                        {
                          term: "Area",
                          description: experiment.area || "N/A",
                        },
                        {
                          term: "ID",
                          description: String(experiment.id),
                        },
                        {
                          term: "Created",
                          description: new Date(
                            experiment.created_at,
                          ).toLocaleDateString(),
                        },
                      ]}
                    />
                  </BlockStack>
                </Box>
              </Card>

              <InlineGrid
                columns={{ xs: 1, sm: 2, md: 2, lg: 4, xl: 4 }}
                gap="400"
              >
                <Card>
                  <Box padding="400">
                    <Text variant="headingMd" as="h3">
                      Control Users
                    </Text>
                    <Text variant="headingLg" as="p">
                      {controlVariant?.user_count?.toLocaleString() || "0"}
                    </Text>
                  </Box>
                </Card>
                <Card>
                  <Box padding="400">
                    <Text variant="headingMd" as="h3">
                      Treatment Users
                    </Text>
                    <Text variant="headingLg" as="p">
                      {treatmentVariant?.user_count?.toLocaleString() || "0"}
                    </Text>
                  </Box>
                </Card>
                <Card>
                  <Box padding="400">
                    <Tooltip
                      content="Percentage of users who converted from the control variant."
                      hasUnderline
                    >
                      <Text as="span" variant="bodyLg" fontWeight="bold">
                        Control CR
                      </Text>
                    </Tooltip>
                    <Text variant="headingLg" as="p">
                      {controlConversionStat
                        ? `${(controlConversionStat.conversion_rate * 100).toFixed(1)}%`
                        : "--%"}
                    </Text>
                    <Text as="p" tone="subdued">
                      Conversions:{" "}
                      {controlConversionStat?.total_conversion_events?.toLocaleString() ||
                        "N/A"}
                    </Text>
                  </Box>
                </Card>
                <Card>
                  <Box padding="400">
                    <Tooltip
                      content="Percentage of users who converted from the treatment variant."
                      hasUnderline
                    >
                      <Text as="span" variant="bodyLg" fontWeight="bold">
                        Treatment CR
                      </Text>
                    </Tooltip>
                    <Text variant="headingLg" as="p">
                      {treatmentConversionStat
                        ? `${(treatmentConversionStat.conversion_rate * 100).toFixed(1)}%`
                        : "--%"}
                    </Text>
                    <Text as="p" tone="subdued">
                      Conversions:{" "}
                      {treatmentConversionStat?.total_conversion_events?.toLocaleString() ||
                        "N/A"}
                    </Text>
                  </Box>
                </Card>
              </InlineGrid>

              <ExperimentAnalyticsChart />
            </BlockStack>
          </Layout.Section>
        </Layout>
      </Page>
    </Frame>
  );
}
