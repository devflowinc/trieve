import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { chatRevenueQuery } from "app/queries/analytics/chat";

export const ChatRevenue = ({
  filters,
  direct,
  granularity,
}: {
  filters: RAGAnalyticsFilter;
  direct: boolean;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    chatRevenueQuery(trieve, filters, granularity, direct),
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
      label={direct ? "Average Chat Revenue (Direct)" : "Average Chat Revenue"}
      tooltipContent={direct ? "The average revenue directly generated from user chat requests." : "The average revenue indirectly generated from user chat requests."}
    />
  );
};
