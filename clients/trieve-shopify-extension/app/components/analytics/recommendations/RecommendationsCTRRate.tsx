import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RecommendationAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { recommendationsCTRRateQuery } from "app/queries/analytics/recommendation";

export const RecommendationsCTRRate = ({
  filters,
  granularity,
}: {
  filters: RecommendationAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    recommendationsCTRRateQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.total_ctr}
      graphData={data?.points}
      granularity={granularity}
      date_range={filters.date_range}
      dataType="percentage"
      xAxis={"time_stamp"}
      yAxis={"ctr"}
      label="CTR Rate"
      tooltipContent="The rate at which users click on products after being recommended."
    />
  );
};
