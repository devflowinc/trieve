import { Card, Box, Text } from "@shopify/polaris";
import { AnalyticsChart } from "./AnalyticsChart";
import { useLoaderData } from "@remix-run/react";
import { loader } from "app/routes/app._dashboard.experimentview.$experimentId";

const eventNameMap = {
  "site-checkout_end": "Checkout",
  "site-add_to_cart": "Add to Cart",
  Click: "Clicked Product",
};

export const ExperimentAnalyticsChart = () => {
  const { treatmentEventsResult, experiment } = useLoaderData<typeof loader>();

  const chartData = treatmentEventsResult
    .reduce(
      (acc, event) => {
        const existing = acc.find(
          (item) => item.event_name === event.event_name,
        );
        const currentEventCount = Number(event.event_count);

        if (existing) {
          if (event.treatment_name === experiment.control_name) {
            existing.control_count = currentEventCount;
          } else if (event.treatment_name === experiment.t1_name) {
            existing.treatment_count = currentEventCount;
          }
        } else {
          acc.push({
            event_name: event.event_name,
            control_count:
              event.treatment_name === experiment.control_name
                ? currentEventCount
                : 0,
            treatment_count:
              event.treatment_name === experiment.t1_name
                ? currentEventCount
                : 0,
          });
        }
        return acc;
      },
      [
        {
          event_name: "Click",
          control_count: 0,
          treatment_count: 0,
        },
        {
          event_name: "site-add_to_cart",
          control_count: 0,
          treatment_count: 0,
        },
        {
          event_name: "site-checkout_end",
          control_count: 0,
          treatment_count: 0,
        },
      ],
    )
    .map((item) => ({
      name: eventNameMap[item.event_name as keyof typeof eventNameMap],
      NoTrieve: item.control_count,
      Trieve: item.treatment_count,
    }));

  const yAxes = [
    {
      key: "NoTrieve" as const,
      label: "No Trieve",
      color: "rgba(54, 162, 235, 0.8)",
    },
    {
      key: "Trieve" as const,
      label: "Trieve",
      color: "rgba(75, 192, 192, 0.8)",
    },
  ];

  return (
    <Card>
      <Box padding="400">
        <Text variant="headingMd" as="h2">
          Event Comparison
        </Text>
        <Box minHeight="300px" paddingBlockStart="400">
          <AnalyticsChart
            data={chartData}
            xAxis="name"
            yAxes={yAxes}
            chartType="bar"
            granularity="day"
            wholeUnits={true}
            yAxisLabel="Event Count"
            yAxisSuggestedMax={10}
          />
        </Box>
      </Box>
    </Card>
  );
};
