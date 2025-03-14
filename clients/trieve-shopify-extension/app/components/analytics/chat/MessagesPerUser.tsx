import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter, TopicAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { messagesPerUserQuery } from "app/queries/analytics/chat";

export const MessagesPerUser = ({
  filters,
  granularity,
}: {
  filters: TopicAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    messagesPerUserQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_messages_per_user}
      graphData={data?.points}
      granularity={granularity}
      date_range={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"messages_per_user"}
      label="Messages Per User"
      tooltipContent="The average number of messages a user sends in one session."
    />
  );
};
