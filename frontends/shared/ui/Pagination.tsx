import { Accessor } from "solid-js";
import { IoChevronBack, IoChevronForward } from "solid-icons/io";

interface PaginationProps {
  pages: {
    page: Accessor<number>;
    nextPage: () => void;
    prevPage: () => void;
    canGoNext: Accessor<boolean>;
  };
  total_pages?: number;
  perPage?: number;
}
export const Pagination = (props: PaginationProps) => {
  return (
    <nav
      class="flex flex-col items-center justify-between bg-white pt-3 sm:px-6"
      aria-label="Pagination">
      <div class="flex flex-1 justify-between gap-3 sm:justify-end">
        <button
          onClick={() => props.pages.prevPage()}
          disabled={props.pages.page() === 1}
          class="relative inline-flex items-center gap-1 rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus-visible:outline-offset-0 disabled:cursor-default disabled:opacity-70 disabled:hover:bg-white">
          <IoChevronBack /> Previous
        </button>
        <button
          onClick={() => props.pages.nextPage()}
          disabled={!props.pages.canGoNext()}
          class="relative inline-flex items-center gap-1 rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus-visible:outline-offset-0 disabled:cursor-default disabled:opacity-70 disabled:hover:bg-white">
          Next
          <IoChevronForward />
        </button>
      </div>
      {props.perPage && props.total_pages ? (
        <div class="hidden sm:block pt-1">
          <p class="text-sm text-gray-700">
            Showing
            <span class="px-1 font-medium">
              {(props.pages.page() - 1) * props.perPage || 1}
            </span>
            to
            <span class="px-1 font-medium">
              {(props.pages.page() - 1) * props.perPage + props.perPage}
            </span>
            of
            <span class="px-1 font-medium">
              {props.total_pages * props.perPage}
            </span>
            results
          </p>
        </div>
      ) : null}
    </nav>
  );
};
