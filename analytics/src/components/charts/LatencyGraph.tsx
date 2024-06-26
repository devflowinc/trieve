import { createQuery } from "@tanstack/solid-query";
import { ChartCard } from "./ChartCard";
import { AnalyticsParams } from "shared/types";
import { useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getLatency } from "../../api/latency";

interface LatencyGraphProps {
  filters: AnalyticsParams;
}
export const LatencyGraph = (props: LatencyGraphProps) => {
  const dataset = useContext(DatasetContext);
  const latencyQuery = createQuery(() => ({
    queryKey: [
      "latency",
      { filters: props.filters, dataset: dataset().dataset.id },
    ],
    queryFn: async () => {
      return await getLatency(props.filters, dataset().dataset.id);
    },
  }));

  return <ChartCard width={3}>{JSON.stringify(latencyQuery.data)}</ChartCard>;
};
