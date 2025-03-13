import { Box, Card, Text } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { searchUsageQuery } from "app/queries/analytics/search";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";

export const SearchUsageChart = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(searchUsageQuery(trieve, filters, granularity));

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.total_searches}
      graphData={data?.usage_points}
      granularity={granularity}
      date_range={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"requests"}
      label="Search Usage"
      tooltipContent="The total number of searches made by users."
    />
  );
};
