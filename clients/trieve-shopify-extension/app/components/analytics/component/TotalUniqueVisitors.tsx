import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { totalUniqueUsersQuery } from "app/queries/analytics/component";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";

export const TotalUniqueVisitors = ({
  filters,
  granularity,
}: {
  filters: ComponentAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    totalUniqueUsersQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.total_unique_users}
      graphData={data?.points}
      granularity={granularity}
      xAxis={"time_stamp"}
      yAxes={[{ key: "point", label: "Total Unique Visitors" }]}
      dateRange={filters.date_range}
      tooltipContent="The total number of unique visitors to your store that interacted with the Trieve component."
    />
  );
};
