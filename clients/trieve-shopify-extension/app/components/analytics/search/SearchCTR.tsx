import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchCTRQuery } from "app/queries/analytics/search";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";

export const SearchCTRChart({
  filters,
  granularity,
}: {
  filters: ComponentAnalyticsFilter;
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
      date_range={filters.date_range}
      dataType="percentage"
      xAxis={"time_stamp"}
      yAxis={"ctr"}
      label="CTR Rate"
      tooltipContent="The rate at which users click on products within the chat sessions."
    />
  );
}
