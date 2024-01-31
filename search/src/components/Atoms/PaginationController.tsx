import { useLocation, useNavigate } from "@solidjs/router";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { Show, For, createEffect, createSignal } from "solid-js";

export const createArrayWithCenteredRange = (center: number, range: number) => {
  const array = [];
  const indicesBeforeCenter = Math.floor(range / 2);

  if (center === Math.floor(range / 2) + 1) {
    for (let j = 1; j <= range; j++) {
      array.push(j);
    }
  } else {
    let currentValue = Math.max(1, center - indicesBeforeCenter);

    for (let j = 0; j < range; j++) {
      array.push(currentValue);
      currentValue++;
    }
  }

  return array;
};

interface PaginationControllerProps {
  page: number;
  totalPages: number;
}

export const PaginationController = (props: PaginationControllerProps) => {
  const navigate = useNavigate();
  const location = useLocation();

  const [url, setUrl] = createSignal<URL | null>(null);

  createEffect(() => {
    const curUrl =
      "https://" + window.location.host + location.pathname + location.search;
    setUrl(new URL(curUrl));
  });

  return (
    <>
      <Show when={props.page != 1}>
        <button
          onClick={() => {
            if (!url()) return;

            url()?.searchParams.set("page", "${props.page - 1}");
            navigate(url()?.toString() ?? "");
          }}
        >
          <BiRegularChevronLeft class="h-8 w-8 fill-current text-neutral-400 dark:text-neutral-500" />
        </button>
      </Show>
      <For
        each={createArrayWithCenteredRange(
          // Center on the current page, unless the current page is the last or second to last page
          props.totalPages - props.page > 1 ? props.page : props.totalPages - 2,
          // Show 5 pages, unless there are less than 5 total pages
          Math.min(props.totalPages, 5),
        )}
      >
        {(n) => (
          <button
            classList={{
              "flex h-8 w-8 items-center justify-center rounded-full focus:bg-neutral-400/70 dark:focus:bg-neutral-500/80":
                true,
              "bg-neutral-400/70 dark:bg-neutral-500/80": n === props.page,
              "bg-neutral-200 dark:bg-neutral-700": n !== props.page,
            }}
            onClick={() => {
              if (!url) return;

              url()?.searchParams.set("page", n.toString());
              navigate(url()?.toString() ?? "");
            }}
          >
            {n}
          </button>
        )}
      </For>
      <Show when={props.page < props.totalPages}>
        <button
          onClick={() => {
            if (!url) return;

            url()?.searchParams.set("page", `${props.page + 1}`);
            navigate(url()?.toString() ?? "");
          }}
        >
          <BiRegularChevronRight class="h-8 w-8 fill-current text-neutral-400 dark:text-neutral-500" />
        </button>
      </Show>
    </>
  );
};
