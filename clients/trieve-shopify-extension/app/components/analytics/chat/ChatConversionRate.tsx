import { Box, Card, Text, Spinner } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter, Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { chatConversionRateQuery } from "app/queries/analytics/chat";

export const ChatConversionRate = ({
  filters,
  granularity,
}: {
  filters: RAGAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    chatConversionRateQuery(trieve, filters, granularity),
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
      yAxis={"point"}
      label="Chat Conversion Rate"
      tooltipContent="The percentage of chat sessions that led to cart additions or purchases."
    />
  );
};
