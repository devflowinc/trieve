import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import {
  RAGAnalyticsFilter,
  SearchAnalyticsFilter,
  TopicAnalyticsFilter,
} from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchesPerUserQuery } from "app/queries/analytics/search";

export const SearchesPerUser = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    searchesPerUserQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_searches_per_user}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      xAxis={"time_stamp"}
      yAxes={[{ key: "point", label: "Searches Per User" }]}
      tooltipContent="The average number of searches a user performs in one session."
    />
  );
};
