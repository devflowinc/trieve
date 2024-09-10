import { flexRender, type Table as TableType } from "@tanstack/solid-table";
import { Pagination } from "./Pagination";
import { For } from "solid-js";
import { cn } from "shared/utils";
import { usePagination } from "../hooks/usePagination";

type TableProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  table: TableType<any>;
  pages: ReturnType<typeof usePagination>;
  total?: number;
  perPage?: number;
};

export const Table = (props: TableProps) => {
  return (
    <>
      <table class="min-w-full border-separate border-spacing-0">
        <thead>
          <For each={props.table.getHeaderGroups()}>
            {(headerGroup) => (
              <tr>
                <For each={headerGroup.headers}>
                  {(header) => (
                    <th class="sticky top-0 z-10 border-b border-neutral-300 bg-white bg-opacity-75 py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter sm:pl-6 lg:pl-8">
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
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
              <tr>
                <For each={row.getVisibleCells()}>
                  {(cell) => (
                    <td
                      class={cn(
                        idx() !== props.table.getRowModel().rows.length - 1
                          ? "border-b border-neutral-200"
                          : "",
                        "whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 sm:pl-6 lg:pl-8",
                      )}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </td>
                  )}
                </For>
              </tr>
            )}
          </For>
        </tbody>
      </table>
      <Pagination
        pages={props.pages}
        perPage={props.perPage}
        total={props.total}
      />
    </>
  );
};
