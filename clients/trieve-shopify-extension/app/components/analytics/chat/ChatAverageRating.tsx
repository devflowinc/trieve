import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { RAGAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { chatAverageRatingQuery } from "app/queries/analytics/chat";

export const ChatAverageRating = ({
  filters,
  granularity,
}: {
  filters: RAGAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    chatAverageRatingQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_chat_rating}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"point"}
      label="Average Chat Rating"
      tooltipContent="The average rating that users give to the chat."
    />
  );
};
