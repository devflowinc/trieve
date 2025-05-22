import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { SearchAnalyticsFilter, Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchConversionRateQuery } from "app/queries/analytics/search";

export const SearchConversionRate = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    searchConversionRateQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.conversion_rate}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      dataType="percentage"
      xAxis={"time_stamp"}
      yAxes={[{ key: "point", label: "Search Conversion Rate" }]}
      tooltipContent="The percentage of searches that led to cart additions or purchases."
    />
  );
};
