import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter, TopicAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { topicsCTRRateQuery } from "app/queries/analytics/chat";

export const TopicCTRRate = ({
  filters,
  granularity,
}: {
  filters: TopicAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    topicsCTRRateQuery(trieve, filters, granularity),
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
      tooltipContent="The rate at which users click on products within the chat sessions."
    />
  );
};
