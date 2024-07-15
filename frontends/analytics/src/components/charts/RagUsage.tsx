import { createQuery } from "@tanstack/solid-query";
import { RAGAnalyticsFilter } from "shared/types";
import { getRAGUsage } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { Show, useContext } from "solid-js";
import { ChartCard } from "./ChartCard";

interface RagUsageProps {
  filter: RAGAnalyticsFilter;
}
export const RagUsage = (props: RagUsageProps) => {
  const dataset = useContext(DatasetContext);
  const ragUsageQuery = createQuery(() => ({
    queryKey: ["rag-usage", { filter: props.filter }],
    queryFn: async () => {
      return getRAGUsage(dataset().dataset.id, props.filter);
    },
  }));

  return (
    <ChartCard title="RAG Usage" width={1}>
      <Show
        fallback={<div class="py-6 text-center">Loading...</div>}
        when={ragUsageQuery.data}
      >
        {(data) => (
          <div class="py-4 text-center">
            <span class="pr-1 text-3xl">{data().total_queries}</span>
            <span class="text-sm opacity-80">RAG Queries</span>
          </div>
        )}
      </Show>
    </ChartCard>
  );
};
