/* eslint-disable solid/reactivity */
import { RagQueryEvent, SearchQueryEvent } from "shared/types";
import { createEffect, createSignal, Show, useContext } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useBetterNav } from "../../utils/useBetterNav";
import {
  createSolidTable,
  getCoreRowModel,
  SortingState,
  Table,
} from "@tanstack/solid-table";
import { Card } from "../../components/charts/Card";
import { DatasetContext } from "../../../contexts/DatasetContext";
import {
  ALL_FAKE_RAG_OPTIONS,
  RagAnalyticsGraphs,
} from "../../components/RagAnalyticsGraphs";
import {
  RAGSortByCols,
  useDataExplorerRag,
} from "../../hooks/data/useDataExplorerRag";
import { RAGFilterBar } from "../../components/RAGFilterBar";
import { getRAGQueries } from "../../api/analytics";
import { IoOpenOutline } from "solid-icons/io";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";

export const RAGAnalyticsPage = () => {
  const navigate = useBetterNav();
  const datasetContext = useContext(DatasetContext);

  const {
    pages,
    ragTableQuery,
    sortBy,
    setSortBy,
    filters,
    setFilters,
    sortOrder,
    setSortOrder,
  } = useDataExplorerRag();
  const [sorting, setSorting] = createSignal<SortingState>([
    {
      id: sortBy(),
      desc: sortOrder() === "desc",
    },
  ]);

  createEffect(() => {
    setSortBy(sorting()[0].id as RAGSortByCols);
    setSortOrder(sorting()[0].desc ? "desc" : "asc");
  });

  const columns: SortableColumnDef<RagQueryEvent>[] = [
    {
      accessorKey: "user_message",
      header: "User Message",
    },
    {
      accessorKey: "created_at",
      header: "Queried At",
      sortable: true,
      cell(props) {
        return format(
          parseCustomDateString(props.getValue<string>()),
          "M/d/yy h:mm a",
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
                navigate(
                  `/dataset/${datasetContext.datasetId()}/analytics/rag/${
                    props.row.id
                  }`,
                );
              }}
            >
              <IoOpenOutline />
              {props.getValue<RagQueryEvent["results"]>().length}
            </button>
          </Show>
        );
      },
    },
  ];

  const table = createSolidTable({
    get data() {
      return ragTableQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
      get sorting() {
        return sorting();
      },
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    manualSorting: true,
    enableSortingRemoval: false,
    onSortingChange: setSorting,
  });

  return (
    <div>
      <div class="mt-4 pb-1 text-lg">All RAG Queries</div>
      <div class="rounded-md bg-white">
        <Show when={ragTableQuery.data}>
          <Card>
            <RAGFilterBar noPadding filters={filters} setFilters={setFilters} />
            <div class="mt-4 overflow-x-auto">
              <TanStackTable
                pages={pages}
                perPage={10}
                table={table as unknown as Table<SearchQueryEvent>}
                onRowClick={(row: SearchQueryEvent) =>
                  navigate(
                    `/dataset/${datasetContext.datasetId()}/analytics/rag/${
                      row.id
                    }`,
                  )
                }
                exportFn={(page: number) =>
                  getRAGQueries({
                    datasetId: datasetContext.datasetId(),
                    filter: filters,
                    page: page,
                    sort_by: sortBy(),
                    sort_order: sortOrder(),
                  })
                }
              />
              <Show when={ragTableQuery.data?.length === 0}>
                <div class="py-8 text-center">No Data.</div>
              </Show>
            </div>
          </Card>
        </Show>
      </div>
      <div class="my-4 border-b border-b-neutral-200 pt-2" />
      <RagAnalyticsGraphs />
    </div>
  );
};
