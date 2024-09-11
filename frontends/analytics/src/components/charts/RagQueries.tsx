import { RagQueryEvent } from "shared/types";
import {
  getCoreRowModel,
  createSolidTable,
  SortingState,
} from "@tanstack/solid-table";
import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  Show,
} from "solid-js";
import { Card } from "./Card";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { ALL_FAKE_RAG_OPTIONS } from "../../pages/RagAnalyticsPage";
import { FullScreenModal, JSONMetadata } from "shared/ui";
import { IoOpenOutline } from "solid-icons/io";
import { RagQueriesProps, useRagData } from "../../hooks/data/useRagData";

export const RagQueries = (props: RagQueriesProps) => {
  const { ragQueriesQuery, pages, sortOrder, setSortOrder, usage } =
    useRagData(props);
  const [open, setOpen] = createSignal<boolean>(false);
  const [current, setCurrent] = createSignal<number>(0);
  const [sorting, setSorting] = createSignal<SortingState>([
    {
      id: "created_at",
      desc: sortOrder() === "desc",
    },
  ]);

  createEffect(() => {
    setSortOrder(sorting()[0].desc ? "desc" : "asc");
  });

  const columns: Accessor<SortableColumnDef<RagQueryEvent>[]> = createMemo(
    () => [
      {
        accessorKey: "user_message",
        header: "User Message",
      },
      {
        accessorKey: "created_at",
        header: "Searched At",
        sortable: true,
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
            <Show
              when={props.getValue<RagQueryEvent["results"]>().length}
              fallback={props.getValue<RagQueryEvent["results"]>().length}
            >
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
            </Show>
          );
        },
      },
    ],
  );

  return (
    <Card
      title="RAG Queries"
      subtitle={"All RAG messages (topic/message and generate_from_chunk)."}
      class="flex flex-col justify-between px-4"
      width={2}
    >
      <Show
        fallback={<div class="py-8 text-center">Loading...</div>}
        when={ragQueriesQuery.data}
      >
        {(data) => {
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
              <TanStackTable
                pages={pages}
                perPage={10}
                total={usage?.data?.total_queries}
                table={createSolidTable({
                  get data() {
                    return ragQueriesQuery.data || [];
                  },
                  state: {
                    pagination: {
                      pageIndex: pages.page(),
                      pageSize: 10,
                    },
                  },
                  columns: columns(),
                  getCoreRowModel: getCoreRowModel(),
                  manualPagination: true,
                })}
              />
            </>
          );
        }}
      </Show>
    </Card>
  );
};
