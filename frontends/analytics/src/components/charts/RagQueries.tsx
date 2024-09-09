import { RAGAnalyticsFilter, RAGSortBy, SortOrder } from "shared/types";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, Show, useContext } from "solid-js";
import { getRAGQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { ChartCard } from "./ChartCard";
import { Select, Table, Td, Th, Tr } from "shared/ui";
import { ALL_FAKE_RAG_OPTIONS } from "../../pages/RagAnalyticsPage";

interface RagQueriesProps {
  filter: RAGAnalyticsFilter;
}

const ALL_SORT_BY: RAGSortBy[] = ["created_at", "latency", "top_score"];
const ALL_SORT_ORDER: SortOrder[] = ["asc", "desc"];

export const RagQueries = (props: RagQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  const [sortOrder, setSortOrder] = createSignal<SortOrder>("asc");

  createEffect(() => {
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "rag-queries",
        {
          page: curPage + 1,
          filter: props.filter,
          sortOrder: sortOrder(),
        },
      ],
      queryFn: async () => {
        const results = await getRAGQueries({
          datasetId,
          page: curPage + 1,
          filter: props.filter,
          sort_order: sortOrder(),
        });
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const ragQueriesQuery = createQuery(() => ({
    queryKey: [
      "rag-queries",
      {
        page: pages.page(),
        filter: props.filter,
        sortOrder: sortOrder(),
      },
    ],
    queryFn: () => {
      return getRAGQueries({
        datasetId: dataset().dataset.id,
        page: pages.page(),
        filter: props.filter,
      });
    },
  }));

  return (
    <ChartCard
      title="RAG Queries"
      subtitle={"All RAG messages (topic/message and generate_from_chunk)."}
      class="flex flex-col justify-between px-4"
      width={2}
      controller={
        <div class="flex gap-2">
          <Select
            class="min-w-[110px] bg-neutral-100/90"
            options={ALL_SORT_ORDER}
            display={formatSortOrder}
            selected={sortOrder()}
            onSelected={(e) => setSortOrder(e)}
          />
        </div>
      }
    >
      <div>
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={ragQueriesQuery.data}
        >
          {(data) => (
            <Table
              fallback={<div class="py-8 text-center">No Data</div>}
              headerClass="px-2"
              class="my-4"
              headers={<Tr><Th>Message</Th><Th>RAG Type</Th><Th>Results</Th></Tr>}
              data={data()}
            >
              {(row) => (
                <Tr>
                  <Td fullWidth={true}>{row.user_message}</Td>
                  <Td class="min-w-[100px]">{ALL_FAKE_RAG_OPTIONS.find(rag => rag.value === row.rag_type)?.label || row.rag_type}</Td>
                  <Td class="text-center">{row.results.length}</Td>
                </Tr>
              )}
            </Table>
          )}
        </Show>
      </div>
    </ChartCard>
  );
};

const formatSortBy = (sortBy: RAGSortBy) => {
  switch (sortBy) {
    case "created_at":
      return "Created At";
    case "latency":
      return "Latency";
    case "top_score":
      return "Top Score";
    default:
      return sortBy;
  }
};

const formatSortOrder = (sortOrder: SortOrder) => {
  switch (sortOrder) {
    case "asc":
      return "Ascending";
    case "desc":
      return "Descending";
    default:
      return sortOrder;
  }
};
