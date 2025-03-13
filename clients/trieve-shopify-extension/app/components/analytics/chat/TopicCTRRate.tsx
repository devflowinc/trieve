import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { topicsCTRRateQuery } from "app/queries/analytics/chat";

export const TopicCTRRate = ({
    filters,
    granularity,
}: {
    filters: ComponentAnalyticsFilter;
    granularity: Granularity;
}) => {
    const { trieve } = useTrieve();
    const { data } = useQuery(topicsCTRRateQuery(trieve, filters, granularity));

    return (
        <GraphComponent
            topLevelMetric={data?.total_ctr}
            graphData={data?.ctr_points}
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
