import {
  flexRender,
  type Table as TableType,
  type ColumnDef,
} from "@tanstack/solid-table";
import { Accessor, For, Show } from "solid-js";
import { cn } from "shared/utils";
import { Pagination } from "shared/ui";
import { FaSolidAngleDown, FaSolidAngleUp } from "solid-icons/fa";

export type SortableColumnDef<TValue> = ColumnDef<any, TValue> & {
  sortable?: boolean;
};

type TableProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  table: TableType<any>;
  small?: boolean;
  pages: {
    page: Accessor<number>;
    nextPage: () => void;
    prevPage: () => void;
    canGoNext: Accessor<boolean>;
  };
  total?: number;
  perPage?: number;
  classNames?: string;
  onRowClick?: (row: any) => void;
};

export const TanStackTable = (props: TableProps) => {
  return (
    <>
      <table
        class={cn(
          "min-w-full border-separate border-spacing-0",
          props.classNames ?? ""
        )}
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
                        "sticky top-0 z-10 border-b border-neutral-300 bg-white bg-opacity-75 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter sm:pl-6 lg:pl-8"
                      )}
                    >
                      {(header.column.columnDef as SortableColumnDef<any>)
                        .sortable ? (
                        <button
                          class="flex items-center gap-1"
                          onClick={header.column.getToggleSortingHandler()}
                        >
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext()
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
                                header.getContext()
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
                        "whitespace-nowrap text-sm font-medium text-neutral-900"
                      )}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </td>
                  )}
                </For>
              </tr>
            )}
          </For>
        </tbody>
      </table>
      {props.pages.canGoNext() || props.pages.page() !== 1 ? (
        <Pagination
          pages={props.pages}
          perPage={props.perPage}
          total={props.total}
        />
      ) : null}
    </>
  );
};
