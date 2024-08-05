import { AiFillCaretLeft } from "solid-icons/ai";
import { usePagination } from "../hooks/usePagination";

interface PaginationButtonsProps {
  pages: ReturnType<typeof usePagination>;
  size?: number;
}
export const PaginationButtons = (props: PaginationButtonsProps) => {
  return (
    <div class="flex items-center justify-center">
      <button
        disabled={props.pages.page() === 1}
        onClick={() => props.pages.prevPage()}
      >
        <AiFillCaretLeft
          classList={{
            "opacity-40": props.pages.page() === 1,
          }}
          size={props.size || 24}
        />
      </button>
      <div class="w-[18px] text-center">{props.pages.page()}</div>
      <button
        disabled={props.pages.page() === props.pages.maxPageDiscovered()}
        onClick={() => props.pages.nextPage()}
      >
        <AiFillCaretLeft
          classList={{
            "opacity-40":
              props.pages.page() === props.pages.maxPageDiscovered(),
            "transform rotate-180": true,
          }}
          size={props.size || 24}
        />
      </button>
    </div>
  );
};
