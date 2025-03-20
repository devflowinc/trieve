/* eslint-disable prefer-const */
import { createQuery } from "@tanstack/solid-query";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { Show, useContext } from "solid-js";
import { getLatency } from "../../api/analytics";

import "chartjs-adapter-date-fns";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { AnalyticsChart } from "./AnalyticsChart";

interface LatencyGraphProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

export const LatencyGraph = (props: LatencyGraphProps) => {
  const dataset = useContext(DatasetContext);
  const latencyQuery = createQuery(() => ({
    queryKey: ["latency", { params: props.params, dataset: dataset.datasetId }],
    queryFn: async () => {
      return await getLatency(
        props.params.filter,
        props.params.granularity,
        dataset.datasetId(),
      );
    },
  }));

  return (
    <Show when={latencyQuery.data}>
      {(data) => (
        <AnalyticsChart
          data={data()}
          granularity={props.params.granularity}
          date_range={props.params.filter.date_range}
          yLabel="Latency (ms)"
          yAxis="point"
          xAxis="time_stamp"
        />
      )}
    </Show>
  );
};
