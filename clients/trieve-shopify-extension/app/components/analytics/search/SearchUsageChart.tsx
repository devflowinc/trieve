import { Box, Card, Text } from "@shopify/polaris";
import { BarChart } from "@shopify/polaris-viz";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import {
  parseCustomDateString,
  queryStateToChartState,
} from "app/queries/analytics/formatting";
import { searchUsageQuery } from "app/queries/analytics/search";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";

export const SearchUsageChart = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, status } = useQuery(
    searchUsageQuery(trieve, filters, granularity),
  );

  return (
    <Card>
      <Text as="h5" variant="headingSm">
        Search Usage
      </Text>
      <Box minHeight="14px"></Box>
      <BarChart
        xAxisOptions={{
          allowLineWrap: true,
        }}
        state={queryStateToChartState(status)}
        showLegend={false}
        data={[
          {
            name: "Search Usage",
            color: "purple",
            metadata: {},
            // @ts-expect-error undocumented date formatting feature: will update with custom format anyways
            data: (data?.usage_points || []).map((point) => ({
              key: parseCustomDateString(point.time_stamp),
              value: point.requests,
            })),
          },
        ]}
      ></BarChart>
    </Card>
  );
};
