import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { recommendationsUsageQuery } from "app/queries/analytics/recommendation";
import { RecommendationAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";

export const RecommendationsUsageChart = ({
  filters,
  granularity,
}: {
  filters: RecommendationAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    recommendationsUsageQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.total_requests}
      graphData={data?.points}
      granularity={granularity}
      date_range={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"requests"}
      label="Recommendations Usage"
      tooltipContent="The total number of recommendations made by users."
    />
  );
};
