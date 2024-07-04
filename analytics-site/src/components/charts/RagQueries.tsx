import { RagQueryEvent } from "shared/types";
import { ChartCard } from "./ChartCard";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, For, Show, useContext } from "solid-js";
import { getRAGQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";

export const RagQueries = () => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect(() => {
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: ["RAG-queries", { page: curPage + 1 }],
      queryFn: async () => {
        const results = await getRAGQueries(datasetId, curPage + 1);
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { page: pages.page() }],
    queryFn: () => {
      return getRAGQueries(dataset().dataset.id, pages.page());
    },
  }));

  return (
    <ChartCard class="px-4" width={5}>
      <div class="text-lg">RAG Queries</div>
      <div class="text-sm text-neutral-600">
        All RAG messages (topic/message and generate_from_chunk).
      </div>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <div class="py-2">
            <For each={data()}>
              {(rag_query_event) => {
                return <RagQueryEventCard rag_query_event={rag_query_event} />;
              }}
            </For>
          </div>
        )}
      </Show>
      <div class="flex justify-end">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </ChartCard>
  );
};

interface QueryCardProps {
  rag_query_event: RagQueryEvent;
}
const RagQueryEventCard = (props: QueryCardProps) => {
  return (
    <div class="flex justify-between">
      <div class="truncate">{props.rag_query_event.user_message}</div>
      <div>{props.rag_query_event.rag_type}</div>
    </div>
  );
};
