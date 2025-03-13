import { Box, Card, Text, Tooltip } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { totalUniqueUsersQuery } from "app/queries/analytics/component";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { AnalyticsChart } from "../AnalyticsChart";
import { GraphComponent } from "../GraphComponent";

export const TotalUniqueVisitors = ({
  filters,
  granularity,
}: {
  filters: ComponentAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data } = useQuery(totalUniqueUsersQuery(trieve, filters, granularity));

  return (
    <GraphComponent
      topLevelMetric={data?.total_unique_users}
      graphData={data?.time_points}
      granularity={granularity}
      xAxis={"time_stamp"}
      yAxis={"unique_users"}
      dateRange={filters.date_range}
      label="Total Unique Visitors"
      tooltipContent="The total number of unique visitors to your store that interacted with the Trieve component."
    />
  );
};
