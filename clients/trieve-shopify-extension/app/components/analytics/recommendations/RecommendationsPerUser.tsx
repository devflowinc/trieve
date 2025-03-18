import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RecommendationAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { recommendationsPerUserQuery } from "app/queries/analytics/recommendation";

export const RecommendationsPerUser = ({
  filters,
  granularity,
}: {
  filters: RecommendationAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    recommendationsPerUserQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_recommendations_per_user}
      graphData={data?.points}
      granularity={granularity}
      date_range={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"recommendations_per_user"}
      label="Recommendations Per User"
      tooltipContent="The average number of recommendations a user receives in one session."
    />
  );
};
