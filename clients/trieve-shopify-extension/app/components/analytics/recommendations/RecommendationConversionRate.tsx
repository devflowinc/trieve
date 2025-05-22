import { Box, Card, Text, Spinner } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RecommendationAnalyticsFilter, Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { recommendationConversionRateQuery } from "app/queries/analytics/recommendation";

export const RecommendationConversionRate = ({
  filters,
  granularity,
}: {
  filters: RecommendationAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    recommendationConversionRateQuery(trieve, filters, granularity),
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
      yAxes={[{ key: "point", label: "Recommendation Conversion Rate" }]}
      tooltipContent="The percentage of recommendations that led to cart additions or purchases."
    />
  );
};
