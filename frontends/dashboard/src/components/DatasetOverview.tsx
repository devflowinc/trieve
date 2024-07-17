/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { TbDatabasePlus } from "solid-icons/tb";
import {
  Show,
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
  AiOutlineBarChart,
  AiOutlineClear,
  AiOutlineComment,
  AiOutlineSearch,
} from "solid-icons/ai";
import { formatDate } from "../formatters";
import { TbReload } from "solid-icons/tb";
import { createToast } from "./ShowToasts";
import { Tooltip } from "shared/ui";
import { DatasetAndUsage, DefaultError, Organization } from "shared/types";
import { Table, Td, Th, Tr } from "shared/ui";

export interface DatasetOverviewProps {
  setOpenNewDatasetModal: Setter<boolean>;
  selectedOrganization: Accessor<Organization>;
}

export const DatasetOverview = (props: DatasetOverviewProps) => {
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

  const orgDatasetParams = (datasetId: string) => {
    const orgId = props.selectedOrganization().id;
    return orgId && datasetId
      ? `/?organization=${orgId}&dataset=${datasetId}`
      : "";
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
      <Show when={(maxDatasets() === 0 && page() === 0) || !hasLoaded()}>
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
      <Show when={maxDatasets() > 0 && hasLoaded()}>
        <div class="mt-8">
          <div class="overflow-hidden rounded shadow ring-1 ring-black ring-opacity-5">
            <Table
              headers={
                <Tr>
                  <Th>Name</Th>
                  <Th>Tools</Th>
                  <Th>Chunk Count</Th>
                  <Th>ID</Th>
                  <Th>Created</Th>
                  <Th class="sr-only">Delete</Th>
                  <Th />
                </Tr>
              }
              class="min-w-full"
              data={datasets()}
            >
              {(dataset) => (
                <DatasetTableRow
                  orgDatasetParams={orgDatasetParams(dataset.dataset.id)}
                  dataset={dataset}
                  clearDataset={clearDataset}
                  deleteDataset={deleteDataset}
                  reloadChunkCount={reloadChunkCount}
                  usage={usage}
                />
              )}
            </Table>
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

interface DatasetTableRowProps {
  dataset: DatasetAndUsage;
  orgDatasetParams: string;
  deleteDataset: (datasetId: string) => Promise<void>;
  clearDataset: (datasetId: string) => Promise<void>;
  usage: Accessor<
    Record<
      string,
      {
        chunk_count: number;
      }
    >
  >;
  reloadChunkCount: (datasetId: string) => void;
}

const DatasetTableRow = (props: DatasetTableRowProps) => {
  const analyticsUiURL = import.meta.env.VITE_ANALYTICS_UI_URL as string;
  const searchUiURL = import.meta.env.VITE_SEARCH_UI_URL as string;
  const chatUiURL = import.meta.env.VITE_CHAT_UI_URL as string;
  const navigate = useNavigate();
  return (
    <Tr class="cursor-pointer hover:bg-neutral-100">
      <Td
        onClick={() => {
          navigate(`/dashboard/dataset/${props.dataset.dataset.id}/start`);
        }}
      >
        {props.dataset.dataset.name}
      </Td>
      <Td
        onClick={() => {
          navigate(`/dashboard/dataset/${props.dataset.dataset.id}/start`);
        }}
      >
        <div class="flex items-center gap-4">
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              window.open(`${searchUiURL}${props.orgDatasetParams}`);
            }}
            class="hover:text-fuchsia-500"
            title="Open search playground for this dataset"
          >
            <AiOutlineSearch class="h-5 w-5" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              window.open(`${chatUiURL}${props.orgDatasetParams}`);
            }}
            class="hover:text-fuchsia-500"
            title="Open RAG playground for this dataset"
          >
            <AiOutlineComment class="h-5 w-5" />{" "}
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              window.open(`${analyticsUiURL}${props.orgDatasetParams}`);
            }}
            class="hover:text-fuchsia-500"
            title="Open analytics for this dataset"
          >
            <AiOutlineBarChart class="h-5 w-5" />
          </button>
        </div>
      </Td>
      <Td
        onClick={() => {
          navigate(`/dashboard/dataset/${props.dataset.dataset.id}/start`);
        }}
      >
        <span class="inline-flex items-center">
          {props.usage()[props.dataset.dataset.id]?.chunk_count ??
            props.dataset.dataset_usage.chunk_count}{" "}
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              props.reloadChunkCount(props.dataset.dataset.id);
            }}
            class="pl-2 hover:text-fuchsia-500"
          >
            <TbReload />
          </button>
        </span>
      </Td>
      <Td
        class="hidden lg:block"
        onClick={() => {
          navigate(`/dashboard/dataset/${props.dataset.dataset.id}/start`);
        }}
      >
        {props.dataset.dataset.id}
      </Td>
      <Td class="">{formatDate(new Date(props.dataset.dataset.created_at))}</Td>
      <Td>
        <button
          class="text-neutral-500 hover:text-neutral-900"
          onClick={() => {
            navigate(`/dashboard/dataset/${props.dataset.dataset.id}/settings`);
          }}
        >
          <FaSolidGear />
        </button>
        <button
          class="text-red-500 hover:text-neutral-900"
          onClick={() => {
            confirm("Are you sure you want to delete this dataset?") &&
              void props.deleteDataset(props.dataset.dataset.id);
          }}
        >
          <FiTrash />
        </button>
        <button
          class="text-red-500 hover:text-neutral-900"
          onClick={() => {
            confirm("Are you sure you want to clear this dataset?") &&
              void props.clearDataset(props.dataset.dataset.id);
          }}
        >
          <AiOutlineClear />
        </button>
      </Td>
    </Tr>
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
