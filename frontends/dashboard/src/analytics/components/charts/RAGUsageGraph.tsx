import { createQuery } from "@tanstack/solid-query";
import { AnalyticsParams, RAGAnalyticsFilter } from "shared/types";
import { Show, useContext } from "solid-js";
import { getRAGUsage, getRagUsageGraph } from "../../api/analytics";

interface RAGUsageProps {
  params: {
    filter: RAGAnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

import { Card } from "./Card";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { AnalyticsChart } from "./AnalyticsChart";

export const RAGUsageGraph = (props: RAGUsageProps) => {
  const dataset = useContext(DatasetContext);
  const usageQuery = createQuery(() => ({
    queryKey: [
      "rag-usage-graph",
      { params: props.params, dataset: dataset.datasetId() },
    ],
    queryFn: async () => {
      return await getRagUsageGraph(
        props.params.filter,
        props.params.granularity,
        dataset.datasetId(),
      );
    },
  }));

  const ragTotalQuery = createQuery(() => ({
    queryKey: ["rag-usage", { filter: props.params }],
    queryFn: () => {
      return getRAGUsage(dataset.datasetId(), props.params.filter);
    },
  }));

  return (
    <Card
      width={2}
      controller={
        <Show when={ragTotalQuery.data}>
          {(total) => (
            <div class="text-sm">{total().total_queries} Total Queries</div>
          )}
        </Show>
      }
      title="RAG Usage"
    >
      <Show when={usageQuery.data}>
        <>
          <AnalyticsChart
            data={usageQuery.data}
            granularity={props.params.granularity}
            date_range={props.params.filter.date_range}
            yLabel="Requests"
            wholeUnits
            yAxis="point"
            xAxis="time_stamp"
          />
        </>
      </Show>
    </Card>
  );
};
