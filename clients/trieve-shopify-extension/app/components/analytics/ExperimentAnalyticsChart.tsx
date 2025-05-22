import { Card, Box, Text, InlineGrid, BlockStack } from "@shopify/polaris";
import { AnalyticsChart } from "./AnalyticsChart";
import { useLoaderData } from "@remix-run/react";
import { loader } from "app/routes/app._dashboard.experimentview.$experimentId";

interface SingleEventChartData {
  categoryName: string;
  noTrieveCount: number;
  trieveCount: number;
}

const eventNameMap = {
  "site-checkout_end": "Checkout",
  "site-add_to_cart": "Add to Cart",
  Click: "Clicked Product",
};

export const ExperimentAnalyticsChart = () => {
  const { treatmentEventsResult, experiment } = useLoaderData<typeof loader>();

  const individualChartsData = treatmentEventsResult.reduce(
    (acc, event) => {
      const eventNameKey =
        eventNameMap[event.event_name as keyof typeof eventNameMap];
      let chartObj = acc.find((c) => c.categoryName === eventNameKey);

      if (!chartObj) {
        return acc;
      }

      const currentEventCount = Number(event.event_count);
      if (event.treatment_name === experiment.control_name) {
        chartObj.noTrieveCount += currentEventCount;
      } else if (event.treatment_name === experiment.t1_name) {
        chartObj.trieveCount += currentEventCount;
      }

      return acc;
    },
    [
      {
        categoryName: "Clicked Product",
        noTrieveCount: 0,
        trieveCount: 0,
      },
      {
        categoryName: "Add to Cart",
        noTrieveCount: 0,
        trieveCount: 0,
      },
      {
        categoryName: "Checkout",
        noTrieveCount: 0,
        trieveCount: 0,
      },
    ] as SingleEventChartData[],
  );

  const yAxesConfig = [
    {
      key: "noTrieveCount" as const,
      label: "Without Trieve",
      color: "rgba(54, 162, 235, 0.8)",
    }, // Blue
    {
      key: "trieveCount" as const,
      label: "With Trieve",
      color: "rgba(75, 192, 192, 0.8)",
    }, // Teal
  ];

  return (
    <Card>
      <Box padding="400">
        <Text variant="headingMd" as="h2">
          Event Breakdown
        </Text>
        {individualChartsData.length === 0 && (
          <Box paddingBlockStart="400">
            <Text as="p" alignment="center" tone="subdued">
              No event data available for this experiment.
            </Text>
          </Box>
        )}
        <InlineGrid
          columns={{
            xs: 1,
            sm: 1,
            md: 3,
          }}
          gap="400"
        >
          {individualChartsData.map((eventData) => {
            const controlCount = eventData.noTrieveCount;
            const treatmentCount = eventData.trieveCount;
            const delta = treatmentCount - controlCount;

            let deltaDisplay = "";
            let percentageDisplay = "";
            let tone: "success" | "critical" | "subdued" = "subdued";

            if (controlCount === 0 && treatmentCount > 0) {
              deltaDisplay = `+ ${treatmentCount}`;
              percentageDisplay = "(from 0)";
              tone = "success";
            } else if (controlCount > 0 && treatmentCount === 0) {
              deltaDisplay = `-${controlCount}`;
              percentageDisplay = "(-100%)";
              tone = "critical";
            } else if (controlCount === 0 && treatmentCount === 0) {
              deltaDisplay = "+0";
              percentageDisplay = "(+0.0%)";
              tone = "subdued";
            } else if (controlCount > 0) {
              const percentage = (delta / controlCount) * 100;
              deltaDisplay = `${delta >= 0 ? "+" : ""}${delta}`;
              percentageDisplay = ` (${percentage >= 0 ? "+" : ""}${percentage.toFixed(1)}%)`;
              if (delta > 0) tone = "success";
              else if (delta < 0) tone = "critical";
            } else {
              // Fallback for any other scenarios, like controlCount being negative (not for counts)
              deltaDisplay = `${delta >= 0 ? "+" : ""}${delta}`;
              if (delta > 0) tone = "success";
              else if (delta < 0) tone = "critical";
            }
            const fullDeltaText = `${deltaDisplay} ${percentageDisplay}`.trim();

            return (
              <Box key={eventData.categoryName} paddingBlockStart="200">
                <BlockStack gap="100" align="center" inlineAlign="center">
                  <Text variant="bodyLg" as="p" alignment="center">
                    {eventData.categoryName.charAt(0).toUpperCase() +
                      eventData.categoryName.slice(1)}
                  </Text>
                  <Text as="p" variant="bodyMd" tone={tone} alignment="center">
                    {fullDeltaText}
                  </Text>
                </BlockStack>
                <Box>
                  <AnalyticsChart
                    data={[eventData]}
                    xAxis="categoryName"
                    yAxes={yAxesConfig}
                    chartType="bar"
                    granularity="day"
                    wholeUnits={true}
                    yAxisLabel={`# of ${eventData.categoryName}s`}
                    yAxisSuggestedMax={10}
                  />
                </Box>
              </Box>
            );
          })}
        </InlineGrid>
      </Box>
    </Card>
  );
};
