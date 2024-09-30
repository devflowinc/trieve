/* eslint-disable @typescript-eslint/no-explicit-any */
import { AnalyticsParams, CTRSearchQuery } from "shared/types";
import { Accessor, createMemo, createSignal, Show, useContext } from "solid-js";
import {
  FullScreenModal,
  JsonInput,
  SortableColumnDef,
  TanStackTable,
} from "shared/ui";
import { IoOpenOutline } from "solid-icons/io";
import { Card } from "./Card";
import { useBetterNav } from "../../utils/useBetterNav";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { MagicSuspense } from "../../../components/MagicBox";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { useCTRSearchQueries } from "../../hooks/data/useCTRSearchQueries";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";

interface CTRSearchQueriesProps {
  params: AnalyticsParams;
  width?: number;
}

export const CTRSearchQueries = (props: CTRSearchQueriesProps) => {
  const datasetContext = useContext(DatasetContext);
  const [current, setCurrent] = createSignal<number>(0);
  const [showClickedChunk, setShowClickedChunk] = createSignal(false);
  const navigate = useBetterNav();
  const { pages, searchCTRQueriesQuery, queryFn } = useCTRSearchQueries({
    // eslint-disable-next-line solid/reactivity
    params: props.params,
  });

  const columns: Accessor<SortableColumnDef<CTRSearchQuery>[]> = createMemo(
    () => {
      return [
        {
          accessorKey: "created_at",
          header: "Timestamp",
          cell(props) {
            return format(
              // eslint-disable-next-line solid/reactivity
              parseCustomDateString(props.getValue<string>()),
              "M/d/yy h:mm a",
            );
          },
        },
        {
          accessorKey: "query",
          header: "Query",
        },
        {
          accessorKey: "clicked_chunk",
          header: "Clicked Chunk",
          cell(props) {
            return (
              <button
                class="flex items-center gap-2 text-left"
                onClick={() => {
                  if (!showClickedChunk()) {
                    setShowClickedChunk(true);
                    setCurrent(props.row.index);
                  }
                }}
              >
                <IoOpenOutline />
                {props.getValue<CTRSearchQuery["clicked_chunk"]>().chunk.id}
              </button>
            );
          },
        },
        {
          accessorKey: "clicked_chunk",
          header: "Click Position",
          cell(props) {
            // eslint-disable-next-line solid/reactivity
            return props.getValue<CTRSearchQuery["clicked_chunk"]>().position;
          },
        },
        {
          accessorKey: "results",
          header: "All Results",
          cell(props) {
            return (
              <button
                class="flex items-center gap-2 text-left"
                onClick={() => {
                  navigate(
                    `/dataset/${datasetContext.datasetId()}/analytics/query/` +
                      (props.row.original as CTRSearchQuery).request_id,
                  );
                }}
              >
                <IoOpenOutline />
                {props.getValue<CTRSearchQuery["results"]>().length}
              </button>
            );
          },
        },
      ];
    },
  );

  const table = createMemo(() => {
    const currCols = columns();
    const table = createSolidTable({
      get data() {
        return searchCTRQueriesQuery.data || [];
      },
      state: {
        pagination: {
          pageIndex: pages.page(),
          pageSize: 10,
        },
      },
      columns: currCols,
      getCoreRowModel: getCoreRowModel(),
      manualPagination: true,
    });
    return table;
  });

  return (
    <Card
      title={"Searches With Clicks"}
      subtitle={"Searches that have a CTR event associated with them"}
      class="px-4"
      width={props.width || 2}
    >
      <MagicSuspense unstyled skeletonKey="searchCTRqueries">
        <Show
          fallback={<div class="py-8 text-center">No Data.</div>}
          when={searchCTRQueriesQuery.data}
        >
          {(data) => (
            <>
              <FullScreenModal
                title="Chunk"
                class="max-h-[80vh] max-w-[80vw] overflow-y-auto p-3"
                show={showClickedChunk}
                setShow={setShowClickedChunk}
              >
                <Show when={data()[current()].clicked_chunk.chunk}>
                  <JsonInput
                    value={() => data()[current()].clicked_chunk.chunk}
                    class="min-w-[60vw]"
                    readonly
                  />
                </Show>
              </FullScreenModal>
              <TanStackTable
                exportFn={queryFn}
                small
                pages={pages}
                perPage={10}
                table={table()}
              />
            </>
          )}
        </Show>
      </MagicSuspense>
    </Card>
  );
};
