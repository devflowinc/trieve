import { createQuery } from "@tanstack/solid-query";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { Show, useContext } from "solid-js";
import { getRpsUsageGraph } from "../../api/analytics";
import { DatasetContext } from "../../../contexts/DatasetContext";
import "chartjs-adapter-date-fns";
import { AnalyticsChart } from "./AnalyticsChart";

interface SearchUsageProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

export const SearchUsageGraph = (props: SearchUsageProps) => {
  const dataset = useContext(DatasetContext);
  const usageQuery = createQuery(() => ({
    queryKey: [
      "search-usage",
      { params: props.params, dataset: dataset.datasetId() },
    ],
    queryFn: async () => {
      return await getRpsUsageGraph(
        props.params.filter,
        props.params.granularity,
        dataset.datasetId(),
      );
    },
  }));

  return (
    <Show when={usageQuery.data}>
      {(data) => (
        <div class="h-full w-full">
          <AnalyticsChart
            data={data()}
            granularity={props.params.granularity}
            date_range={props.params.filter.date_range}
            yLabel="Requests"
            yAxis="point"
            xAxis="time_stamp"
            xLabel="Timestamp"
          />
        </div>
      )}
    </Show>
  );
};
