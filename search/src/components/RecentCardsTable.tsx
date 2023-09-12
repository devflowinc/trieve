import { For, Show, createEffect, createSignal } from "solid-js";
import type { CardMetadataWithVotes } from "../../utils/apiTypes";
import { FaSolidChevronRight, FaSolidChevronLeft } from "solid-icons/fa";

export interface RecentCardsTableProps {
  recentCards?: CardMetadataWithVotes[];
}

export const RecentCardsTable = (props: RecentCardsTableProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;

  const [errorText, setErrorText] = createSignal("");
  const [recentCards, setRecentCards] = createSignal<CardMetadataWithVotes[]>(
    // eslint-disable-next-line solid/reactivity
    props.recentCards ?? [],
  );
  const [isLoading, setIsLoading] = createSignal(false);

  const [page, setPage] = createSignal(1);

  createEffect(() => {
    const curPage = page();
    const abortController = new AbortController();
    setIsLoading(true);

    void fetch(`${apiHost}/recent_cards/${curPage}`, {
      signal: abortController.signal,
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      setIsLoading(false);
      if (!response.ok) {
        setErrorText("Error fetching recent cards");
        return;
      }

      void response.json().then((data) => {
        setRecentCards(data as unknown as CardMetadataWithVotes[]);
      });
    });

    return () => {
      setIsLoading(false);
      abortController.abort();
    };
  });

  return (
    <div class="mx-auto w-full max-w-4xl px-4 sm:px-8 md:px-20">
      <div class="rounded bg-neutral-100 p-4 dark:bg-neutral-700 sm:p-6 lg:p-8">
        <div class="sm:flex sm:items-center">
          <div class="sm:flex-auto">
            <h1 class="text-base font-semibold leading-6 text-neutral-900 dark:text-neutral-100">
              Recently Created Cards
            </h1>
            <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-200">
              A list of the most recently created cards.
            </p>
            <Show when={errorText()}>
              <p class="mt-2 text-sm text-red-600 dark:text-red-400">
                {errorText()}
              </p>
            </Show>
          </div>
        </div>
        <div class="mt-2">
          <table
            classList={{
              "min-w-full divide-y divide-neutral-400 dark:divide-neutral-800":
                true,
              "animate-pulse": isLoading(),
            }}
          >
            <thead>
              <tr>
                <th
                  scope="col"
                  class="py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100 sm:pl-0"
                >
                  Content
                </th>
                <th
                  scope="col"
                  class="py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100"
                >
                  Score
                </th>
                <th
                  scope="col"
                  class="hidden py-3.5 pl-2 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100 sm:table-cell"
                >
                  Link
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-300 dark:divide-neutral-600">
              <For each={recentCards()}>
                {(recent_card) => (
                  <tr>
                    <td>
                      <div class="line-clamp-2 p-1 text-sm text-neutral-800 dark:text-neutral-100">
                        <a
                          href={`/card/${recent_card.id}`}
                          class="line-clamp-2 break-all text-turquoise-600 underline dark:text-acid-500 min-[672px]:max-w-[300px]  min-[672px]:break-normal min-[860px]:max-w-none"
                        >
                          {recent_card.content}
                        </a>
                      </div>
                    </td>
                    <td>
                      <div class="line-clamp-1 break-all p-1 text-sm text-neutral-800 dark:text-neutral-100">
                        {recent_card.total_upvotes -
                          recent_card.total_downvotes}
                      </div>
                    </td>
                    <td>
                      <div class="line-clamp-1 hidden min-w-[150px] break-all p-1 text-sm text-neutral-800 dark:text-neutral-100 sm:table-cell sm:min-w-[200px]">
                        <a
                          href={`/card/${recent_card.id}`}
                          class="line-clamp-1 underline"
                        >
                          {/* remove http://www or https://www */}
                          {recent_card.link?.replace(
                            /^(?:https?:\/\/)?(?:www\.)?/i,
                            "",
                          ) ?? ""}
                        </a>
                      </div>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
        <div class="mt-4 flex items-center justify-between">
          <Show when={page() > 1}>
            <button
              classList={{
                "flex items-center space-x-1 rounded-md bg-neutral-100 p-2 px-4 py-2 text-sm dark:bg-neutral-600":
                  true,
                "animate-pulse": isLoading(),
              }}
              disabled={page() === 1 || isLoading()}
              onClick={() => setPage(page() - 1)}
            >
              <FaSolidChevronLeft class="h-4 w-4 fill-current" />
              <span>Previous</span>
            </button>
          </Show>
          <div class="flex-1" />
          <button
            classList={{
              "flex items-center space-x-1 rounded-md bg-neutral-100 p-2 px-4 py-2 text-sm dark:bg-neutral-600":
                true,
              "animate-pulse": isLoading(),
            }}
            disabled={recentCards().length < 5 || isLoading()}
            onClick={() => setPage(page() + 1)}
          >
            <span>Next</span>
            <FaSolidChevronRight class="h-4 w-4 fill-current" />
          </button>
        </div>
      </div>
    </div>
  );
};
