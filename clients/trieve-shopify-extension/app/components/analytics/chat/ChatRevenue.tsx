import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { chatRevenueQuery } from "app/queries/analytics/chat";

export const ChatRevenue = ({
  filters,
  granularity,
}: {
  filters: RAGAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    chatRevenueQuery(trieve, filters, granularity),
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
      yAxis={"point"}
      label="Average Chat Revenue"
      tooltipContent="The average revenue generated from user chat requests."
    />
  );
};
