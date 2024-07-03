/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { TbDatabasePlus } from "solid-icons/tb";
import {
  Show,
  For,
  Setter,
  Accessor,
  createSignal,
  createEffect,
  Switch,
  Match,
  onCleanup,
} from "solid-js";
import { useNavigate } from "@solidjs/router";
import { FiTrash } from "solid-icons/fi";
import { FaSolidGear } from "solid-icons/fa";
import { useDatasetPages } from "../hooks/useDatasetPages";
import {
  AiFillCaretLeft,
  AiFillCaretRight,
  AiOutlineClear,
} from "solid-icons/ai";
import { formatDate } from "../formatters";
import { TbReload } from "solid-icons/tb";
import { createToast } from "./ShowToasts";
import { DefaultError, Organization } from "shared/types";

export interface DatasetOverviewProps {
  setOpenNewDatasetModal: Setter<boolean>;
  selectedOrganization: Accessor<Organization | undefined>;
}

export const DatasetOverview = (props: DatasetOverviewProps) => {
  const navigate = useNavigate();
  const [page, setPage] = createSignal(0);
  const [datasetSearchQuery, setDatasetSearchQuery] = createSignal("");
  const [usage, setUsage] = createSignal<
    Record<string, { chunk_count: number }>
  >({});
  const { datasets, maxPageDiscovered, maxDatasets, removeDataset, hasLoaded } =
    useDatasetPages({
      // eslint-disable-next-line solid/reactivity
      org: props.selectedOrganization,
      searchQuery: datasetSearchQuery,
      page: page,
      setPage,
    });

  createEffect(() => {
    const finishedLoading = hasLoaded();
    if (!finishedLoading) {
      return;
    }

    const api_host = import.meta.env.VITE_API_HOST as unknown as string;
    const newUsage: Record<string, { chunk_count: number }> = {};
    const abortController = new AbortController();

    const fetchUsage = (datasetId: string) => {
      return new Promise((resolve, reject) => {
        fetch(`${api_host}/dataset/usage/${datasetId}`, {
          method: "GET",
          headers: {
            "TR-Dataset": datasetId,
            "Content-Type": "application/json",
          },
          credentials: "include",
        })
          .then((response) => {
            if (!response.ok) {
              reject(new Error("Failed to fetch dataset usage"));
            }
            return response.json();
          })
          .then((data) => {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            newUsage[datasetId] = data;
            resolve(data);
          })
          .catch((error) => {
            console.error(
              `Failed to fetch usage for dataset ${datasetId}:`,
              error,
            );
            reject(error);
          });
      });
    };

    const fetchInitialUsage = () => {
      const promises = datasets().map((dataset) =>
        fetchUsage(dataset.dataset.id),
      );
      Promise.all(promises)
        .then(() => {
          setUsage(newUsage);
        })
        .catch((error) => {
          console.error("Failed to fetch initial usage: ", error);
        });
    };

    fetchInitialUsage();

    onCleanup(() => {
      abortController.abort("Cleanup fetch");
    });
  });

  createEffect(() => {
    props.selectedOrganization();
    setPage(0);
  });

  const deleteDataset = async (datasetId: string) => {
    const api_host = import.meta.env.VITE_API_HOST as unknown as string;
    const response = await fetch(`${api_host}/dataset/${datasetId}`, {
      method: "DELETE",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
    });
    if (response.ok) {
      removeDataset(datasetId);
    } else {
      const error = (await response.json()) as DefaultError;
      createToast({
        title: "Error",
        type: "error",
        message: `Failed to delete dataset: ${error.message}`,
      });
    }
  };

  const clearDataset = async (datasetId: string) => {
    const api_host = import.meta.env.VITE_API_HOST as unknown as string;
    const response = await fetch(`${api_host}/dataset/clear/${datasetId}`, {
      method: "PUT",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
    });

    if (!response.ok) {
      const error = (await response.json()) as DefaultError;
      createToast({
        title: "Error",
        type: "error",
        message: `Failed to clear dataset: ${error.message}`,
      });
    }
  };

  const reloadChunkCount = (datasetId: string) => {
    const api_host = import.meta.env.VITE_API_HOST as unknown as string;
    if (!datasetId) {
      console.error("Dataset ID is undefined.");
      return;
    }

    const currentUsage = usage();

    fetch(`${api_host}/dataset/usage/${datasetId}`, {
      method: "GET",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch dataset usage");
        }
        return response.json();
      })
      .then((newData) => {
        const prevCount = currentUsage[datasetId]?.chunk_count || 0;
        const newCount: number = newData.chunk_count as number;
        const countDifference = newCount - prevCount;

        setUsage((prevUsage) => ({
          ...prevUsage,
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          [datasetId]: newData,
        }));

        createToast({
          title: "Updated",
          type: "success",
          message: `Successfully updated chunk count: ${countDifference} chunk${
            Math.abs(countDifference) === 1 ? " has" : "s have"
          } been ${
            countDifference > 0
              ? "added"
              : countDifference < 0
                ? "removed"
                : "added or removed"
          } since last update.`,
          timeout: 3000,
        });
      })
      .catch((error) => {
        createToast({
          title: "Error",
          type: "error",
          message: `Failed to reload chunk count: ${error}`,
        });
      });
  };

  return (
    <>
      <div class="flex items-center">
        <div class="flex w-full items-end justify-between pt-2">
          <div>
            <div class="flex items-center gap-2">
              <h1 class="text-base font-semibold leading-6">Datasets</h1>
              <Show when={!hasLoaded()}>
                <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
              </Show>
            </div>
            <Show
              fallback={
                maxDatasets() > 0 ? (
                  hasLoaded() ? (
                    <p class="text-sm text-red-700">
                      No datasets match your search query.
                    </p>
                  ) : (
                    <p class="text-sm text-blue-700">
                      Loading datasets... Please wait and try again when
                      finished.
                    </p>
                  )
                ) : (
                  <p class="text-sm text-neutral-700">
                    This organization does not have any datasets.
                  </p>
                )
              }
              when={datasets().length > 0}
            >
              <p class="text-sm text-neutral-700">
                {" "}
                A list of all the datasets{" "}
              </p>
            </Show>
          </div>
          <div class="flex gap-2">
            <input
              value={datasetSearchQuery()}
              onInput={(e) => {
                setPage(0);
                setDatasetSearchQuery(e.currentTarget.value);
              }}
              placeholder="Search datasets..."
              class="rounded border border-neutral-300/80 bg-white px-2 py-1 text-sm placeholder:text-neutral-400"
            />
            <Show when={maxDatasets() != 0}>
              <button
                class="rounded bg-magenta-500 px-3 py-2 text-sm font-semibold text-white"
                onClick={() => props.setOpenNewDatasetModal(true)}
              >
                Create Dataset +
              </button>
            </Show>
          </div>
        </div>
      </div>
      <Show when={maxDatasets() === 0 && page() === 0}>
        <Switch>
          <Match when={hasLoaded()}>
            <button
              onClick={() => props.setOpenNewDatasetModal(true)}
              class="relative block w-full rounded-lg border-2 border-dashed border-neutral-300 p-12 text-center hover:border-neutral-400 focus:outline-none focus:ring-2 focus:ring-magenta-500 focus:ring-offset-2"
            >
              <TbDatabasePlus class="mx-auto h-12 w-12 text-magenta" />
              <span class="mt-2 block font-semibold">Create A New Dataset</span>
            </button>
          </Match>
          <Match when={!hasLoaded()}>
            <div class="flex flex-col items-center justify-center gap-2">
              <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
              <p class="animate-pulse">Loading datasets...</p>
            </div>
          </Match>
        </Switch>
      </Show>
      <Show when={maxDatasets() > 0}>
        <div class="mt-8">
          <div class="overflow-hidden rounded shadow ring-1 ring-black ring-opacity-5">
            <table class="min-w-full divide-y divide-neutral-300">
              <thead class="w-full min-w-full bg-neutral-100">
                <tr>
                  <th
                    scope="col"
                    class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                  >
                    Name
                  </th>
                  <th
                    scope="col"
                    class="px-3 py-3.5 text-left text-sm font-semibold"
                  >
                    Chunk Count
                  </th>
                  <th
                    scope="col"
                    class="hidden w-full px-3 py-3.5 text-left text-sm font-semibold lg:block"
                  >
                    ID
                  </th>
                  <th
                    scope="col"
                    class="py-3.5 text-left text-sm font-semibold"
                  >
                    Created
                  </th>
                  <th class="sr-only">Delete</th>
                  <th />
                </tr>
              </thead>
              <tbody class="divide-y divide-neutral-200 bg-white">
                <For each={datasets()}>
                  {(datasetAndUsage) => (
                    <tr class="cursor-pointer hover:bg-neutral-100">
                      <td
                        class="whitespace-nowrap py-4 pl-6 pr-3 text-sm font-medium"
                        onClick={() => {
                          navigate(
                            `/dashboard/dataset/${datasetAndUsage.dataset.id}/start`,
                          );
                        }}
                      >
                        {datasetAndUsage.dataset.name}
                      </td>
                      <td
                        class="whitespace-nowrap px-3 py-4 text-sm text-neutral-600"
                        onClick={() => {
                          navigate(
                            `/dashboard/dataset/${datasetAndUsage.dataset.id}/start`,
                          );
                        }}
                      >
                        <span class="inline-flex items-center">
                          <div>
                            {usage()[datasetAndUsage.dataset.id]?.chunk_count ??
                              datasetAndUsage.dataset_usage.chunk_count}{" "}
                          </div>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              e.preventDefault();
                              reloadChunkCount(datasetAndUsage.dataset.id);
                            }}
                            class="ml-2 hover:text-fuchsia-500"
                          >
                            <TbReload />
                          </button>
                        </span>
                      </td>
                      <td
                        class="hidden whitespace-nowrap px-3 py-4 text-sm text-neutral-600 lg:block"
                        onClick={() => {
                          navigate(
                            `/dashboard/dataset/${datasetAndUsage.dataset.id}/start`,
                          );
                        }}
                      >
                        {datasetAndUsage.dataset.id}
                      </td>
                      <td class="whitespace-nowrap py-4 text-sm text-neutral-600">
                        {formatDate(
                          new Date(datasetAndUsage.dataset.created_at),
                        )}
                      </td>
                      <td class="flex items-center justify-end gap-4 whitespace-nowrap py-4 pr-2 text-right text-sm font-medium">
                        <button
                          class="text-lg text-neutral-500 hover:text-neutral-900"
                          onClick={() => {
                            navigate(
                              `/dashboard/dataset/${datasetAndUsage.dataset.id}/settings`,
                            );
                          }}
                        >
                          <FaSolidGear />
                        </button>
                        <button
                          class="text-lg text-red-500 hover:text-neutral-900"
                          onClick={() => {
                            confirm(
                              "Are you sure you want to delete this dataset?",
                            ) && void deleteDataset(datasetAndUsage.dataset.id);
                          }}
                        >
                          <FiTrash />
                        </button>
                        <button
                          class="text-lg text-red-500 hover:text-neutral-900"
                          onClick={() => {
                            confirm(
                              "Are you sure you want to clear this dataset?",
                            ) && void clearDataset(datasetAndUsage.dataset.id);
                          }}
                        >
                          <AiOutlineClear />
                        </button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
            <Show when={maxPageDiscovered() > 1}>
              <PaginationArrows
                page={page}
                setPage={setPage}
                maxPageDiscovered={maxPageDiscovered}
              />
            </Show>
          </div>
        </div>
      </Show>
    </>
  );
};

const PaginationArrows = (props: {
  page: Accessor<number>;
  setPage: Setter<number>;
  maxPageDiscovered: Accessor<number | null>;
}) => {
  return (
    <div class="flex items-center justify-end gap-2 border-t border-t-neutral-200 p-1">
      <button
        onClick={() => props.setPage((page) => page - 1)}
        disabled={props.page() === 0}
        class="p-2 text-lg font-semibold text-neutral-600 disabled:opacity-50"
      >
        <AiFillCaretLeft />
      </button>
      <div class="text-sm">Page {props.page() + 1}</div>
      <button
        onClick={() => props.setPage((page) => page + 1)}
        disabled={props.page() === props.maxPageDiscovered()}
        class="p-2 text-lg font-semibold text-neutral-600 disabled:opacity-50"
      >
        <AiFillCaretRight />
      </button>
    </div>
  );
};
