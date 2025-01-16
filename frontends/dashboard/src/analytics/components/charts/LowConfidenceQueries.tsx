/* eslint-disable @typescript-eslint/no-explicit-any */
import { AnalyticsParams, SearchQueryEvent } from "shared/types";
import { createSignal, Show, useContext } from "solid-js";
import { FullScreenModal, SortableColumnDef, TanStackTable } from "shared/ui";
import { IoOpenOutline } from "solid-icons/io";
import { Card } from "./Card";
import { useBetterNav } from "../../utils/useBetterNav";
import { useLowConfidenceQueries } from "../../hooks/data/useLowConfidenceQueries";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";
import { MagicSuspense } from "../../../components/MagicBox";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { SearchQueryEventModal } from "../SearchQueryEventModal";

interface LowConfidenceQueriesProps {
  params: AnalyticsParams;
  width?: number;
}

const columns: SortableColumnDef<SearchQueryEvent>[] = [
  {
    accessorKey: "query",
    header: "Query",
  },
  {
    accessorKey: "top_score",
    header: "Score",
    cell(props) {
      // eslint-disable-next-line solid/reactivity
      return props.getValue<number>().toFixed(5);
    },
  },
];

export const LowConfidenceQueries = (props: LowConfidenceQueriesProps) => {
  const datasetContext = useContext(DatasetContext);
  const [thresholdText, setThresholdText] = createSignal("");
  const [open, setOpen] = createSignal(false);
  const [current, setCurrent] = createSignal<SearchQueryEvent | null>(null);
  const navigate = useBetterNav();
  const { pages, lowConfidenceQueriesQuery, queryFn } = useLowConfidenceQueries(
    {
      // eslint-disable-next-line solid/reactivity
      params: props.params,
      thresholdText: thresholdText,
    },
  );

  const table = createSolidTable({
    get data() {
      return lowConfidenceQueriesQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
  });

  return (
    <Card
      subtitle="Searches with the lowest top scores"
      title={"Low Confidence Queries"}
      controller={
        <input
          class="mt-1 border-neutral-800 px-2 text-end text-sm outline-none ring-0 active:border-b-2"
          type="text"
          placeholder="Enter threshold.."
          value={thresholdText()}
          onInput={(e) => setThresholdText(e.currentTarget.value)}
        />
      }
      class="px-4"
      width={props.width || 2}
    >
      <MagicSuspense unstyled skeletonKey="lowconfidencequeries">
        <Show
          fallback={<div class="py-8 text-center">No Data.</div>}
          when={
            lowConfidenceQueriesQuery.data &&
            lowConfidenceQueriesQuery.data.length
          }
        >
          <Show when={current()}>
            {(data) => (
              <FullScreenModal
                title={data().query}
                show={open}
                setShow={setOpen}
                icon={
                  <button
                    type="button"
                    class="hover:text-fuchsia-500"
                    onClick={() => {
                      navigate(
                        `/dataset/${datasetContext.datasetId()}/analytics/query/` +
                          data().id,
                      );
                    }}
                  >
                    <IoOpenOutline />
                  </button>
                }
              >
                <SearchQueryEventModal searchEvent={data()} />
              </FullScreenModal>
            )}
          </Show>
          <TanStackTable
            exportFn={queryFn}
            small
            pages={pages}
            perPage={10}
            table={table}
            onRowClick={(row) => {
              setCurrent(row as any);
              setOpen(true);
            }}
          />
        </Show>
      </MagicSuspense>
    </Card>
  );
};
