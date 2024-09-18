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
import { DatasetAndUsage } from "trieve-ts-sdk";
import { TanStackTable } from "shared/ui";
import { CopyButton } from "./CopyButton";
import { formatDate } from "../utils/formatters";

const colHelp = createColumnHelper<DatasetAndUsage>();

export const DatasetOverview = () => {
  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);
  const userContext = useContext(UserContext);

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

  const table = createMemo(() => {
    const curUsage = usage();

    const columns = [
      colHelp.accessor("dataset.name", {
        header: "Name",
      }),
      colHelp.display({
        header: "Chunk Count",
        cell(info) {
          return curUsage[info.row.original.dataset.id]?.chunk_count ?? 0;
        },
      }),

      colHelp.accessor("dataset.id", {
        header: "ID",
        cell(props) {
          return (
            <div class="flex gap-1">
              {props.row.original.dataset.id}
              <CopyButton text={props.row.original.dataset.id} />
            </div>
          );
        },
      }),
      colHelp.accessor("dataset.created_at", {
        header: "Created",
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
      <div class="flex items-center">
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
