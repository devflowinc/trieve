import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { componentInteractionTimeQuery } from "app/queries/analytics/component";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";

export const AverageInteractionTime = ({
  filters,
  granularity,
}: {
  filters: ComponentAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    componentInteractionTimeQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_interaction_time}
      graphData={data?.points}
      granularity={granularity}
      xAxis={"time_stamp"}
      yAxis={"point"}
      dataType="time"
      dateRange={filters.date_range}
      label="Average Interaction Time"
      tooltipContent="The average time a user spends interacting with the Trieve component."
    />
  );
};
