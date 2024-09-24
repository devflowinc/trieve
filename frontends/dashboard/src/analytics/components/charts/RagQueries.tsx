import {
  AnalyticsParams,
  RAGAnalyticsFilter,
  RagQueryEvent,
  SortOrder,
} from "shared/types";
import {
  getCoreRowModel,
  ColumnDef,
  createSolidTable,
} from "@tanstack/solid-table";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  Show,
  useContext,
} from "solid-js";
import { getRAGQueries, getRAGUsage } from "../../api/analytics";
import { usePagination } from "../../hooks/usePagination";
import { Select, TanStackTable } from "shared/ui";
import { ALL_FAKE_RAG_OPTIONS } from "../../pages/RagAnalyticsPage";
import { FullScreenModal, JSONMetadata } from "shared/ui";
import { IoOpenOutline } from "solid-icons/io";
import { Card } from "./Card";
import { DatasetContext } from "../../../contexts/DatasetContext";

interface RagQueriesProps {
  filter: RAGAnalyticsFilter;
  granularity: AnalyticsParams["granularity"];
}

const ALL_SORT_ORDER: SortOrder[] = ["asc", "desc"];

export const RagQueries = (props: RagQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();
  const [openUserMessage, setOpenUserMessage] = createSignal<boolean>(false);
  const [openLLMCompletion, setOpenLLMCompletion] =
    createSignal<boolean>(false);
  const [openRagResults, setOpenRagResults] = createSignal<boolean>(false);
  const [current, setCurrent] = createSignal<number>(0);

  const [sortOrder, setSortOrder] = createSignal<SortOrder>("asc");

  createEffect(() => {
    const datasetId = dataset.datasetId();
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
        datasetId: dataset.datasetId(),
        page: pages.page(),
        filter: props.filter,
      });
    },
  }));

  const columns: Accessor<ColumnDef<RagQueryEvent>[]> = createMemo(() => [
    {
      accessorKey: "user_message",
      header: "User Message",
      cell(props) {
        return (
          <button
            class="flex items-center gap-2 text-left"
            onClick={() => {
              setOpenUserMessage(true);
              setCurrent(props.row.index);
            }}
          >
            <IoOpenOutline />
            {props.getValue<string>()}
          </button>
        );
      },
    },
    {
      accessorKey: "rag_type",
      header: "Rag Type",
      cell(props) {
        return (
          <>
            {ALL_FAKE_RAG_OPTIONS.find(
              (rag) => rag.value === props.getValue<string>(),
            )?.label || props.getValue<string>()}
          </>
        );
      },
    },
    {
      accessorKey: "llm_response",
      header: "LLM Response",
      cell(props) {
        return (
          <button
            class="flex items-center gap-2 text-left"
            onClick={() => setOpenLLMCompletion(true)}
          >
            <IoOpenOutline />
            {props.getValue<RagQueryEvent["llm_response"]>()}
          </button>
        );
      },
    },
    {
      accessorKey: "results",
      id: "results",
      header: "Results",
      cell(props) {
        return (
          <Show
            when={props.getValue<RagQueryEvent["results"]>().length}
            fallback={props.getValue<RagQueryEvent["results"]>().length}
          >
            <button
              class="flex items-center gap-2 text-left"
              onClick={() => {
                setOpenRagResults(true);
                setCurrent(props.row.index);
              }}
            >
              <IoOpenOutline />
              {props.getValue<RagQueryEvent["results"]>().length}
            </button>
          </Show>
        );
      },
    },
  ]);

  const table = createMemo(() => {
    const curColumns = columns();

    const table = createSolidTable({
      get data() {
        return ragQueriesQuery.data || [];
      },
      state: {
        pagination: {
          pageIndex: pages.page(),
          pageSize: 10,
        },
      },
      columns: curColumns,
      getCoreRowModel: getCoreRowModel(),
      manualPagination: true,
    });

    return table;
  });

  const usage = createQuery(() => ({
    queryKey: ["rag-usage", { filter: props }],
    queryFn: () => {
      return getRAGUsage(dataset.datasetId(), props.filter);
    },
  }));

  return (
    <Card
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
      <Show
        fallback={<div class="py-8 text-center">Loading...</div>}
        when={ragQueriesQuery.data}
      >
        {(data) => {
          return (
            <>
              <FullScreenModal
                show={openRagResults}
                setShow={setOpenRagResults}
                title={`Results found for "${data()[current()].user_message}"`}
              >
                <JSONMetadata
                  monospace
                  copyJSONButton
                  class="text-sm"
                  data={data()[current()].results}
                />
              </FullScreenModal>
              <TanStackTable
                pages={pages}
                perPage={10}
                total={usage?.data?.total_queries}
                table={table()}
              />

              <FullScreenModal
                show={openLLMCompletion}
                setShow={setOpenLLMCompletion}
                title="LLM Completion"
              >
                <p>{data()[current()].llm_response}</p>
              </FullScreenModal>

              <FullScreenModal
                show={openUserMessage}
                setShow={setOpenUserMessage}
                title="Query"
              >
                <p>{data()[current()].user_message}</p>
              </FullScreenModal>
            </>
          );
        }}
      </Show>
    </Card>
  );
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
