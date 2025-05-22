import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { TopicAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { topicsUsageQuery } from "app/queries/analytics/chat";

export const TopicsUsage = ({
  filters,
  granularity,
}: {
  filters: TopicAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    topicsUsageQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.total_topics}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      xAxis={"time_stamp"}
      yAxes={[{ key: "point", label: "Chat Sessions Created" }]}
      tooltipContent="The total number of chat sessions that were created by users."
    />
  );
};
