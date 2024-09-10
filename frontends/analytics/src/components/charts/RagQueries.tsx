import {
  AnalyticsParams,
  RAGAnalyticsFilter,
  RagQueryEvent,
  RAGUsageResponse,
  SortOrder,
} from "shared/types";
import {
  getCoreRowModel,
  ColumnDef,
  createSolidTable,
} from "@tanstack/solid-table";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, Show, useContext } from "solid-js";
import { getRAGQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { ChartCard } from "./ChartCard";
import { Select } from "shared/ui";
import { ALL_FAKE_RAG_OPTIONS } from "../../pages/RagAnalyticsPage";
import { FullScreenModal, JSONMetadata } from "shared/ui";
import { IoOpenOutline } from "solid-icons/io";
import { Table } from "../Table";

interface RagQueriesProps {
  filter: RAGAnalyticsFilter;
  granularity: AnalyticsParams["granularity"];
}

const ALL_SORT_ORDER: SortOrder[] = ["asc", "desc"];

export const RagQueries = (props: RagQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();
  const [open, setOpen] = createSignal<boolean>(false);
  const [current, setCurrent] = createSignal<number>(0);

  const [sortOrder, setSortOrder] = createSignal<SortOrder>("asc");

  createEffect(() => {
    const datasetId = dataset()?.dataset.id;
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

  const defaultColumns: ColumnDef<RagQueryEvent>[] = [
    {
      accessorKey: "user_message",
      header: "User Message",
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
      accessorKey: "results",
      id: "results",
      header: "Results",
      cell(props) {
        return (
          <button
            class="flex items-center gap-2 text-left"
            onClick={() => {
              setOpen(true);
              setCurrent(props.row.index);
            }}
          >
            {props.getValue<RagQueryEvent["results"]>().length}
            <IoOpenOutline />
          </button>
        );
      },
    },
  ];

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
      <Show
        fallback={<div class="py-8 text-center">Loading...</div>}
        when={ragQueriesQuery.data}
      >
        {(data) => {
          const table = createSolidTable({
            get data() {
              return data();
            },
            state: {
              pagination: {
                pageIndex: pages.page(),
                pageSize: 10,
              },
            },
            columns: defaultColumns,
            getCoreRowModel: getCoreRowModel(),
            manualPagination: true,
          });
          const usage = queryClient.getQueryData<RAGUsageResponse>([
            "rag-usage",
            { filter: props },
          ]);
          return (
            <>
              <FullScreenModal
                show={open}
                setShow={setOpen}
                title={`Results found for "${data()[current()].user_message}"`}
              >
                <JSONMetadata
                  monospace
                  copyJSONButton
                  class="text-sm"
                  data={data()[current()].results}
                />
              </FullScreenModal>
              <Table
                pages={pages}
                perPage={10}
                total={usage?.total_queries}
                table={table}
              />
            </>
          );
        }}
      </Show>
    </ChartCard>
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
