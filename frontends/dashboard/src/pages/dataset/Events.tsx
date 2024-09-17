import { For, Show, createEffect, createSignal, useContext } from "solid-js";
import { isEvent, isEventDTO } from "shared/types";
import {
  BiRegularChevronDown,
  BiRegularChevronLeft,
  BiRegularChevronRight,
} from "solid-icons/bi";
import { FiInfo } from "solid-icons/fi";
import { DatasetContext } from "../../contexts/DatasetContext";
import { MultiSelect } from "../../components/MultiSelect";
import { createQuery } from "@tanstack/solid-query";
import { ApiContext } from "../..";
import { EventTypeRequest } from "trieve-ts-sdk";

export const DatasetEvents = () => {
  const { datasetId } = useContext(DatasetContext);
  const trieve = useContext(ApiContext);

  const [page, setPage] = createSignal(1);
  const [pageCount, setPageCount] = createSignal(1);
  const [loading, setLoading] = createSignal(true);
  const [selected, setSelected] = createSignal<
    {
      id: string;
      name: string;
    }[]
  >([]);

  const eventsQuery = createQuery(() => ({
    queryKey: ["events", datasetId],
    refetchInterval: 5000,
    queryFn: async () => {
      const response = await trieve.fetch("/api/events", "post", {
        data: {
          event_types: selected().map((s) => s.id) as EventTypeRequest[],
          page: page(),
        },
        datasetId: datasetId(),
      });
      if (isEventDTO(response)) {
        if (Array.isArray(response.events) && response.events.every(isEvent)) {
          setPageCount(Math.ceil(response.page_count / 10));
          setLoading(false);
          return response.events;
        }
      } else {
        return [];
      }
    },
  }));

  return (
    <div class="mb-3">
      <main class="mx-auto">
        <div class="rounded-md border border-blue-600/20 bg-blue-50 p-4 dark:bg-blue-900">
          <div class="flex space-x-2">
            <FiInfo class="h-5 w-5 text-blue-400 dark:text-blue-50" />
            <p class="text-sm text-blue-700 dark:text-blue-50">
              Events are logged by the server and displayed here for chunk and
              file CRUD operations. You can filter by event type. The list
              refreshes every 5 seconds.
            </p>
          </div>
        </div>
        <div class="mx-auto mt-4 pb-8">
          <div class="">
            <div class="sm:flex sm:items-center">
              <div class="sm:flex-auto">
                <h1 class="text-base font-semibold leading-6 text-gray-900">
                  Events
                </h1>
                <p class="text-sm text-gray-700">
                  Event Log from the server (Refreshes every 5 seconds)
                </p>
              </div>
              <div class="flex min-w-[300px] flex-col gap-1">
                <span class="text-sm">Event Type:</span>
                <MultiSelect
                  items={[
                    {
                      id: "file_uploaded",
                      name: "File Uploaded",
                    },
                    {
                      id: "file_upload_failed",
                      name: "File Upload Failed",
                    },
                    {
                      id: "chunks_uploaded",
                      name: "Chunks Uploaded",
                    },
                    {
                      id: "chunk_updated",
                      name: "Chunk Updated",
                    },
                    {
                      id: "bulk_chunks_deleted",
                      name: "Bulk Chunks Deleted",
                    },
                    {
                      id: "dataset_delete_failed",
                      name: "Dataset Delete Failed",
                    },
                    {
                      id: "qdrant_index_failed",
                      name: "Qdrant Index Failed",
                    },
                    {
                      id: "bulk_chunk_upload_failed",
                      name: "Bulk Chunk Upload Failed",
                    },
                  ]}
                  setSelected={(
                    selected: {
                      id: string;
                      name: string;
                    }[],
                  ) => {
                    setSelected(selected);
                  }}
                />
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
                      <Show when={eventsQuery?.data?.length === 0}>
                        <tr>
                          <td
                            class="px-3 py-4 pt-14 text-center text-sm text-gray-500"
                            colSpan="4"
                          >
                            No events found
                          </td>
                        </tr>
                      </Show>
                      <tbody class="bg-white">
                        <For each={eventsQuery.data}>
                          {(event) => {
                            const [isExpanded, setIsExpanded] =
                              createSignal(false);
                            const [showChevron, setShowChevron] =
                              createSignal(false);

                            let refEl: HTMLDivElement | null = null;

                            createEffect(() => {
                              if (refEl) {
                                if (
                                  refEl.scrollHeight > refEl.clientHeight ||
                                  refEl.scrollWidth > refEl.clientWidth
                                ) {
                                  setShowChevron(true);
                                } else {
                                  setShowChevron(false);
                                }
                              }
                            });

                            return (
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
                                <td
                                  classList={{
                                    "px-3 py-4 text-sm text-gray-500 max-w-lg text-wrap":
                                      true,
                                    "whitespace-normal break-words overflow-ellipsis":
                                      isExpanded(),
                                    "whitespace-nowrap overflow-hidden overflow-ellipsis":
                                      !isExpanded(),
                                  }}
                                >
                                  <div
                                    classList={{
                                      "flex items-start overflow-hidden overflow-ellipsis break-words":
                                        true,
                                      "max-h-10": !isExpanded(),
                                    }}
                                    ref={(el) => {
                                      refEl = el;
                                    }}
                                  >
                                    <Show when={showChevron()}>
                                      <button
                                        onClick={() =>
                                          setIsExpanded(!isExpanded())
                                        }
                                        class="mr-1 focus:outline-none"
                                      >
                                        {isExpanded() ? (
                                          <BiRegularChevronDown class="h-5 w-5 fill-current" />
                                        ) : (
                                          <BiRegularChevronRight class="h-5 w-5 fill-current" />
                                        )}
                                      </button>
                                    </Show>
                                    {JSON.stringify(
                                      JSON.parse(event.event_data),
                                    )}
                                  </div>
                                </td>
                              </tr>
                            );
                          }}
                        </For>
                      </tbody>
                    </table>
                  </Show>
                </div>
              </div>
            </div>
            <Show when={loading()}>
              <div class="flex w-full flex-col items-center justify-center">
                <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
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
