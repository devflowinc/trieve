import {
  flexRender,
  type Table as TableType,
  type Row,
  type ColumnDef,
} from "@tanstack/solid-table";
import { Accessor, createSignal, For, Show } from "solid-js";
import { cn, jsonToCSV } from "shared/utils";
import { Pagination } from "shared/ui";
import { FaSolidAngleDown, FaSolidAngleUp } from "solid-icons/fa";
import { saveAs } from "file-saver";

export type SortableColumnDef<TValue> = ColumnDef<unknown, TValue> & {
  sortable?: boolean;
};

type TableProps<T> = {
  table: TableType<T>;
  small?: boolean;
  pages?: {
    page: Accessor<number>;
    nextPage: () => void;
    prevPage: () => void;
    canGoNext: Accessor<boolean>;
  };
  total_pages?: number;
  perPage?: number;
  class?: string;
  headerClass?: string;
  onRowClick?: (row: Row<T>["original"]) => void;
  exportFn?: (page: number) => Promise<unknown[]>;
};

export const TanStackTable = <T,>(props: TableProps<T>) => {
  const [allData, setAllData] = createSignal<unknown[]>([]);
  const [isCreatingCSV, setIsCreatingCSV] = createSignal<boolean>(false);

  const download = async () => {
    const startDate = +new Date();
    let page = 1;
    if (props.exportFn) {
      setIsCreatingCSV(true);
      // run this loop for a max of 60s
      while (+new Date() < startDate + 60000) {
        const results = await props.exportFn(page);
        if (!results.length) break;
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        setAllData([...allData(), ...results]);
        page = page + 1;
      }
      setIsCreatingCSV(false);
      const csv = jsonToCSV(allData());
      const blob = new Blob([csv], {
        type: "text/plain;charset=utf-8",
      });
      saveAs(blob, "csv-export.csv");
    }
  };
  return (
    <>
      <table
        class={cn("min-w-full border-separate border-spacing-0", props.class)}
      >
        <thead>
          <For each={props.table.getHeaderGroups()}>
            {(headerGroup) => (
              <tr>
                <For each={headerGroup.headers}>
                  {(header) => (
                    <th
                      class={cn(
                        props.small ? "py-2 pl-3 pr-2" : "py-3.5 pl-4 pr-3",
                        "sticky top-0 z-10 border-b border-neutral-300 bg-white bg-opacity-75 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter sm:pl-6 lg:pl-8",
                        props.headerClass,
                      )}
                    >
                      {(header.column.columnDef as SortableColumnDef<unknown>)
                        .sortable ? (
                        <button
                          class="flex items-center gap-1"
                          onClick={header.column.getToggleSortingHandler()}
                        >
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext(),
                              )}
                          <Show when={header.column.getIsSorted() === "desc"}>
                            <FaSolidAngleDown />
                          </Show>
                          <Show when={header.column.getIsSorted() === "asc"}>
                            <FaSolidAngleUp />
                          </Show>
                        </button>
                      ) : (
                        <div>
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext(),
                              )}
                        </div>
                      )}
                    </th>
                  )}
                </For>
              </tr>
            )}
          </For>
        </thead>
        <tbody>
          <For each={props.table.getRowModel().rows}>
            {(row, idx) => (
              <tr
                class={cn({
                  "hover:bg-zinc-400/5 cursor-pointer": props.onRowClick,
                })}
                onClick={() =>
                  props.onRowClick && props.onRowClick(row.original)
                }
              >
                <For each={row.getVisibleCells()}>
                  {(cell) => (
                    <td
                      class={cn(
                        props.small
                          ? "py-3 pl-3 pr-2 sm:pl-4 lg:pl-6"
                          : " py-4 pl-4 pr-3 sm:pl-6 lg:pl-8",
                        idx() !== props.table.getRowModel().rows.length - 1
                          ? "border-b border-neutral-200"
                          : "",
                        "whitespace-nowrap text-sm font-medium text-neutral-900",
                      )}
                    >
                      <span class="max-w-[400px] truncate text-ellipsis block">
                        {" "}
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </span>
                    </td>
                  )}
                </For>
              </tr>
            )}
          </For>
        </tbody>
      </table>
      <div class="flex items-center justify-between pl-4 border-t border-gray-200">
        <Show when={props.exportFn && props.table.getRowCount() > 0}>
          <div class="py-3">
            <button
              onClick={() => void download()}
              class="flex items-center gap-2 hover:bg-neutral-200 rounded-md border bg-neutral-100 px-2 py-1 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-600"
              disabled={isCreatingCSV()}
            >
              {isCreatingCSV() ? "Creating your CSV..." : "Export as CSV"}
            </button>
          </div>
        </Show>
        {props.pages &&
        (props.pages.canGoNext() || props.pages.page() !== 1) ? (
          <Pagination
            pages={props.pages}
            perPage={props.perPage}
            total_pages={props.total_pages}
          />
        ) : null}
      </div>
    </>
  );
};
