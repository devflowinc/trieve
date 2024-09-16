import { AnalyticsFilter, SearchQueryEvent } from "shared/types";
import { createSignal, Show } from "solid-js";
import { FullScreenModal, SortableColumnDef, TanStackTable } from "shared/ui";
import { SearchQueryEventModal } from "../../pages/TrendExplorer";
import { useNoResultsQueries } from "../../hooks/data/useNoResultsQuery";
import { createSolidTable, getCoreRowModel } from "@tanstack/solid-table";

interface NoResultQueriesProps {
  params: {
    filter: AnalyticsFilter;
  };
}

const columns: SortableColumnDef<SearchQueryEvent>[] = [
  {
    accessorKey: "query",
    header: "Query",
    cell(props) {
      return (
        <span class="block max-w-[400px] truncate">
          {props.getValue<string>()}
        </span>
      );
    },
  },
  {
    accessorKey: "latency",
    header: "Latency",
    cell(props) {
      return props.getValue<number>() + "ms";
    },
  },
];

export const NoResultQueries = (props: NoResultQueriesProps) => {
  const [open, setOpen] = createSignal(false);
  const [current, setCurrent] = createSignal<SearchQueryEvent | null>(null);
  const { notResultQuery, pages } = useNoResultsQueries({
    params: props.params,
  });
  const table = createSolidTable({
    get data() {
      return notResultQuery.data || [];
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
    <>
      <div>
        <Show when={notResultQuery.data?.length === 0}>
          <div class="py-8 text-center opacity-80">No Data.</div>
        </Show>
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={notResultQuery.data}
        >
          <Show when={current()}>
            {(data) => (
              <FullScreenModal
                title={data().query}
                show={open}
                setShow={setOpen}
              >
                <SearchQueryEventModal searchEvent={data()} />
              </FullScreenModal>
            )}
          </Show>
          <TanStackTable
            table={table}
            pages={pages}
            perPage={10}
            small
            onRowClick={(row) => {
              setCurrent(row);
              setOpen(true);
            }}
          />
        </Show>
      </div>
    </>
  );
};
