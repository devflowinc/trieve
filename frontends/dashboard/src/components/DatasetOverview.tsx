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
  useContext,
  createMemo,
} from "solid-js";
import { useDatasetPages } from "../hooks/useDatasetPages";
import { AiFillCaretLeft, AiFillCaretRight } from "solid-icons/ai";
import NewDatasetModal from "./NewDatasetModal";
import { UserContext } from "../contexts/UserContext";
import { MagicSuspense } from "./MagicBox";
import { useNavigate } from "@solidjs/router";
import {
  createColumnHelper,
  createSolidTable,
  getCoreRowModel,
} from "@tanstack/solid-table";
import { DatasetAndUsage, DatasetUsageCount } from "trieve-ts-sdk";
import { TanStackTable } from "shared/ui";
import { CopyButton } from "./CopyButton";
import { formatDate } from "../utils/formatters";
import { TbReload } from "solid-icons/tb";
import { createToast } from "../components/ShowToasts";

const colHelp = createColumnHelper<DatasetAndUsage>();

export const DatasetOverview = () => {
  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);
  const userContext = useContext(UserContext) as {
    selectedOrg: () => { id: string };
  };

  const navigate = useNavigate();
  const [page, setPage] = createSignal(0);
  const [datasetSearchQuery, setDatasetSearchQuery] = createSignal("");
  const [usage, setUsage] = createSignal<
    Record<string, { chunk_count: number }>
  >({});
  const { datasets, maxPageDiscovered, maxDatasets, hasLoaded } =
    useDatasetPages({
      org: userContext.selectedOrg().id,
      searchQuery: datasetSearchQuery,
      page: page,
      setPage,
    });

  const refetchChunks = async (datasetId: string) => {
    try {
      const api_host = import.meta.env.VITE_API_HOST as unknown as string;
      const currentUsage = usage();
      const prevCount = currentUsage[datasetId]?.chunk_count || 0;

      const response = await fetch(`${api_host}/dataset/usage/${datasetId}`, {
        method: "GET",
        headers: {
          "TR-Dataset": datasetId,
          "Content-Type": "application/json",
        },
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Failed to fetch dataset usage");
      }

      const newData = (await response.json()) as DatasetUsageCount;
      const newCount = newData.chunk_count;

      const countDifference = newCount - prevCount;

      setUsage((prevUsage) => ({
        ...prevUsage,
        [datasetId]: { chunk_count: newCount },
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
    } catch (_) {
      createToast({
        title: "Error",
        type: "error",
        message: `Failed to reload chunk count.`,
      });
    }
  };

  const table = createMemo(() => {
    const curUsage = usage();

    const columns = [
      colHelp.accessor("dataset.name", {
        header: "Name",
      }),
      colHelp.display({
        header: "Chunk Count",
        cell(info) {
          const datasetId = info.row.original.dataset.id;

          return (
            <div class="flex flex-row content-center items-center gap-1">
              {curUsage[datasetId]?.chunk_count ?? 0}{" "}
              <button
                class="text-sm opacity-80 hover:text-fuchsia-500"
                onClick={(e) => {
                  e.stopPropagation();
                  void refetchChunks(datasetId);
                }}
              >
                <TbReload />
              </button>
            </div>
          );
        },
      }),

      colHelp.accessor("dataset.id", {
        header: "ID",
        cell(props) {
          return (
            <div class="flex gap-2">
              {props.row.original.dataset.id}
              <button
                class="flex flex-row content-center text-sm opacity-80"
                onClick={(e) => {
                  e.stopPropagation();
                }}
              >
                <CopyButton text={props.row.original.dataset.id} />
              </button>
            </div>
          );
        },
      }),
      colHelp.accessor("dataset.created_at", {
        header: "Created At",
        cell(props) {
          // eslint-disable-next-line solid/reactivity
          return formatDate(new Date(props.getValue()));
        },
      }),
    ];

    const table = createSolidTable({
      columns,
      data: datasets(),
      getCoreRowModel: getCoreRowModel(),
    });

    return table;
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
    userContext.selectedOrg().id;
    setPage(0);
  });

  return (
    <>
      <NewDatasetModal
        isOpen={newDatasetModalOpen}
        closeModal={() => {
          setNewDatasetModalOpen(false);
        }}
      />
      <div class="flex items-center py-2">
        <div class="flex w-full items-end justify-between pt-2">
          <div>
            <div class="flex items-center gap-2">
              <h1 class="text-base font-semibold leading-6">Datasets</h1>
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
              <p class="text-sm text-neutral-700">A list of all the datasets</p>
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
                onClick={() => setNewDatasetModalOpen(true)}
              >
                Create Dataset +
              </button>
            </Show>
          </div>
        </div>
      </div>
      <MagicSuspense unstyled>
        <Show when={(maxDatasets() === 0 && page() === 0) || !hasLoaded()}>
          <Switch>
            <Match when={hasLoaded()}>
              <button
                onClick={() => setNewDatasetModalOpen(true)}
                class="relative block w-full rounded-lg border-2 border-dashed border-neutral-300 p-12 text-center hover:border-neutral-400 focus:outline-none focus:ring-2 focus:ring-magenta-500 focus:ring-offset-2"
              >
                <TbDatabasePlus class="mx-auto h-12 w-12 text-magenta" />
                <span class="mt-2 block font-semibold">
                  Create A New Dataset
                </span>
              </button>
            </Match>
          </Switch>
        </Show>
        <Show when={maxDatasets() > 0 && hasLoaded()}>
          <div class="mt-4">
            <Show when={table()}>
              {(table) => (
                <TanStackTable
                  onRowClick={(r) => navigate(`/dataset/${r.dataset.id}`)}
                  class="rounded-md border border-neutral-300 bg-white"
                  headerClass="bg-neutral-200"
                  table={table()}
                />
              )}
            </Show>
            <Show when={maxPageDiscovered() > 1}>
              <PaginationArrows
                page={page}
                setPage={setPage}
                maxPageDiscovered={maxPageDiscovered}
              />
            </Show>
          </div>
        </Show>
      </MagicSuspense>
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
