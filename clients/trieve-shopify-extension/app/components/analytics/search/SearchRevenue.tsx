import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchRevenueQuery } from "app/queries/analytics/search";

export const SearchRevenue = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    searchRevenueQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_revenue}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      dataType="currency"
      xAxis={"time_stamp"}
      yAxes={[{ key: "point", label: "Average Search Revenue" }]}
      tooltipContent="The average revenue generated from user search requests."
    />
  );
};
