import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchCTRQuery } from "app/queries/analytics/search";

export const SearchCTRChart = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    searchCTRQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      topLevelMetric={data?.total_ctr}
      graphData={data?.points}
      loading={isLoading}
      granularity={granularity}
      dateRange={filters.date_range}
      dataType="percentage"
      xAxis={"time_stamp"}
      yAxis={"point"}
      label="Search CTR Rate"
      tooltipContent="The rate at which users click on products within the search sessions."
    />
  );
};
