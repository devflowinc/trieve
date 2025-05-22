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
      topLevelMetric={data?.revenue}
      graphData={data?.points}
      granularity={granularity}
      dateRange={filters.date_range}
      dataType="currency"
      xAxis={"time_stamp"}
      yAxes={[
        {
          key: "point",
          label: direct ? "Direct Chat Revenue" : "Indirect Chat Revenue",
        },
      ]}
      tooltipContent={
        direct
          ? "Sales revenue generated when customers click a Trieve-recommended product and complete the purchase of that recommended item."
          : "These customers messaged through Trieve at least once but but purchased products other than those recommended during their journey."
      }
    />
  );
};
