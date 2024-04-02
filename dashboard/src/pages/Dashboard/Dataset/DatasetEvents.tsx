import {
  For,
  Show,
  createEffect,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { Event, isEvent, isEventDTO } from "../../../types/apiTypes";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";

export const DatasetEvents = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const datasetContext = useContext(DatasetContext);

  const [events, setEvents] = createSignal<Event[]>([]);
  const [page, setPage] = createSignal(1);
  const [pageCount, setPageCount] = createSignal(1);
  const [loading, setLoading] = createSignal(true);

  const getEvents = () => {
    const datasetId = datasetContext.dataset?.()?.id;
    if (!datasetId) return;
    const eventsAbortController = new AbortController();
    void fetch(`${api_host}/events`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": datasetId,
      },
      body: JSON.stringify({
        page: page(),
      }),
      signal: eventsAbortController.signal,
    }).then((res) =>
      res.json().then((data) => {
        if (isEventDTO(data)) {
          if (Array.isArray(data.events) && data.events.every(isEvent)) {
            setEvents(data.events);
            setPageCount(Math.ceil(data.page_count / 10));
            setLoading(false);
          }
        }
      }),
    );
  };

  // Start the interval when the component mounts
  const intervalId = setInterval(() => {
    getEvents();
  }, 5000); // 5000 milliseconds = 5 seconds

  // Cleanup function to clear the interval when the component unmounts
  onCleanup(() => {
    clearInterval(intervalId);
  });

  createEffect(() => {
    getEvents();
  });

  return (
    <div class="mb-3">
      <main class="mx-auto">
        <div class="mx-auto mt-8 pb-8">
          <div class="">
            <div class="sm:flex sm:items-center">
              <div class="sm:flex-auto">
                <h1 class="text-base font-semibold leading-6 text-gray-900">
                  Events
                </h1>
                <p class="mt-2 text-sm text-gray-700">
                  Event Log from the server (Refreshes every 5 seconds)
                </p>
              </div>
            </div>
            <div class="mt-8 flow-root">
              <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
                <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
                  <Show when={!loading()}>
                    <table class="min-w-full max-w-md divide-y divide-gray-300">
                      <thead>
                        <tr>
                          <th
                            scope="col"
                            class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-3"
                          >
                            Level
                          </th>
                          <th
                            scope="col"
                            class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-3"
                          >
                            Event Type
                          </th>
                          <th
                            scope="col"
                            class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                          >
                            Time
                          </th>
                          <th
                            scope="col"
                            class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                          >
                            Metadata
                          </th>
                        </tr>
                      </thead>
                      <tbody class="bg-white">
                        <For each={events()}>
                          {(event) => (
                            <tr class="even:bg-gray-50">
                              <td
                                classList={{
                                  "whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-3":
                                    true,
                                  "text-gray-900":
                                    !event.event_type.includes("failed"),
                                  "text-red-500":
                                    event.event_type.includes("failed"),
                                }}
                              >
                                {event.event_type.includes("failed")
                                  ? "ERROR"
                                  : "INFO"}
                              </td>
                              <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500 sm:pl-3">
                                {event.event_type}
                              </td>
                              <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                                {event.created_at}
                              </td>
                              <td class="text-wrap max-w-md overflow-ellipsis break-words px-3 py-4 text-sm text-gray-500">
                                {JSON.stringify(event.event_data)}
                              </td>
                            </tr>
                          )}
                        </For>
                      </tbody>
                    </table>
                  </Show>
                </div>
              </div>
            </div>
            <Show when={loading()}>
              <div class="flex items-center justify-center">
                <img
                  src="https://media.tenor.com/17U_UB9LwVQAAAAi/loading-response.gif"
                  alt="loading spinner"
                  class="h-12 w-12"
                />
              </div>
            </Show>
            <div class="mt-4 flex items-center justify-between">
              <div />
              <div class="flex items-center">
                <div class="text-sm text-neutral-400">
                  {page()} / {pageCount()}
                </div>
                <button
                  class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                  disabled={page() == 1}
                  onClick={() => {
                    setPage((prev) => prev - 1);
                  }}
                >
                  <BiRegularChevronLeft class="h-6 w-6 fill-current" />
                </button>
                <button
                  class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                  disabled={page() == pageCount()}
                  onClick={() => {
                    setPage((prev) => prev + 1);
                  }}
                >
                  <BiRegularChevronRight class="h-6 w-6 fill-current" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
};
