import {
  Show,
  createEffect,
  createSignal,
  For,
  onMount,
  Setter,
  Accessor,
  Match,
  Switch,
  useContext,
  onCleanup,
} from "solid-js";
import {
  type ChunkGroupDTO,
  type ScoreChunkDTO,
  ChunkBookmarksDTO,
  GroupScoreChunkDTO,
  ChunkFilter,
  MatchCondition,
} from "../../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { PaginationController } from "./Atoms/PaginationController";
import { ConfirmModal } from "./Atoms/ConfirmModal";
import { ScoreChunkArray } from "./ScoreChunkArray";
import { Portal } from "solid-js/web";
import { AiOutlineRobot } from "solid-icons/ai";
import ChatPopup from "./ChatPopup";
import { IoDocumentOutline, IoDocumentsOutline } from "solid-icons/io";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { FaSolidChevronDown, FaSolidChevronUp } from "solid-icons/fa";

export interface Filters {
  tagSet: string[];
  link: string[];
  start: string;
  end: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  metadataFilters: any;
}
export interface ResultsPageProps {
  query: string;
  page: number;
  filters: Filters;
  searchType: string;
  groupUnique?: boolean;
  loading: Accessor<boolean>;
  setLoading: Setter<boolean>;
}

const ResultsPage = (props: ResultsPageProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;

  const [chunkCollections, setChunkCollections] = createSignal<ChunkGroupDTO[]>(
    [],
  );
  const $currentUser = datasetAndUserContext.user;
  const [resultChunks, setResultChunks] = createSignal<ScoreChunkDTO[]>([]);
  const [groupResultChunks, setGroupResultChunks] = createSignal<
    GroupScoreChunkDTO[]
  >([]);
  const [clientSideRequestFinished, setClientSideRequestFinished] =
    createSignal(false);
  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});
  const [bookmarks, setBookmarks] = createSignal<ChunkBookmarksDTO[]>([]);
  const [openChat, setOpenChat] = createSignal(false);
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);
  const [noResults, setNoResults] = createSignal(false);

  const fetchChunkCollections = () => {
    if (!$currentUser?.()) return;
    const dataset = $dataset?.();
    if (!dataset) return;
    void fetch(`${apiHost}/dataset/groups/${dataset.dataset.id}/1`, {
      method: "GET",
      credentials: "include",
      headers: {
        "TR-Dataset": dataset.dataset.id,
      },
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setChunkCollections(data.groups);
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setTotalCollectionPages(data.total_pages);
        });
      }
    });
  };

  const fetchBookmarks = () => {
    const dataset = $dataset?.();
    if (!dataset) return;

    void fetch(`${apiHost}/chunk_group/chunks`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      body: JSON.stringify({
        chunk_ids: resultChunks().flatMap((c) => {
          return c.metadata.map((m) => m.id);
        }),
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const chunkBookmarks = data as ChunkBookmarksDTO[];
          setBookmarks(chunkBookmarks);
        });
      }
    });
  };

  createEffect(() => {
    const dataset = $dataset?.();
    if (!dataset) return;

    const filters: ChunkFilter = {
      must: [],
    };

    if (props.filters.tagSet.length > 0) {
      filters.must = filters.must || [];
      filters.must.push({
        field: "tags",
        match: props.filters.tagSet,
      });
    }
    if (props.filters.link.length > 0) {
      filters.must = filters.must || [];
      filters.must.push({
        field: "link",
        match: props.filters.link,
      });
    }
    if (props.filters.start || props.filters.end) {
      filters.must = filters.must || [];
      filters.must.push({
        field: "timestamp",
        range: {
          gte: Date.parse(
            props.filters.start ? props.filters.start + " 00:00:00" : "null",
          ),
          lte: Date.parse(
            props.filters.end ? props.filters.end + " 00:00:00" : "null",
          ),
        },
      });
    }
    if (props.filters.metadataFilters) {
      filters.must = filters.must || [];
      for (const [key, value] of Object.entries(
        props.filters.metadataFilters as Record<string, unknown>,
      )) {
        filters.must.push({
          field: "metadata." + key,
          match: value as MatchCondition[],
        });
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const requestBody: any = {
      query: props.query,
      page: props.page,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      filters: filters,
      search_type: props.searchType,
      get_collisions: true,
    };

    let searchRoute = "chunk/search";
    const groupUnique = props.groupUnique;
    if (groupUnique) {
      searchRoute = "chunk_group/group_oriented_search";
    }

    props.setLoading(true);

    setGroupResultChunks([]);
    setResultChunks([]);
    setNoResults(false);

    const abortController = new AbortController();

    void fetch(`${apiHost}/${searchRoute}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      signal: abortController.signal,
      credentials: "include",
      body: JSON.stringify(requestBody),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          let resultingChunks: ScoreChunkDTO[] = [];
          if (groupUnique) {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
            const groupResult = data.group_chunks as GroupScoreChunkDTO[];

            setGroupResultChunks(groupResult);

            resultingChunks = groupResult.flatMap((groupChunkDTO) => {
              return groupChunkDTO.metadata;
            });
          } else {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
            resultingChunks = data.score_chunks as ScoreChunkDTO[];

            setResultChunks(resultingChunks);
          }

          if (resultingChunks.length === 0) {
            setNoResults(true);
          }
        });
      }

      setClientSideRequestFinished(true);

      createEffect(() => {
        props.setLoading(false);
      });
    });

    fetchChunkCollections();

    onCleanup(() => {
      abortController.abort();
    });
  });

  onMount(() => {
    fetchBookmarks();
  });

  createEffect(() => {
    if (!openChat()) {
      setSelectedIds((prev) => (prev.length < 10 ? prev : []));
    }
  });

  return (
    <>
      <Show when={openChat()}>
        <Portal>
          <FullScreenModal isOpen={openChat} setIsOpen={setOpenChat}>
            <div class="max-h-[75vh] min-h-[75vh] min-w-[75vw] max-w-[75vw] overflow-y-auto rounded-md scrollbar-thin">
              <ChatPopup
                chunks={resultChunks}
                selectedIds={selectedIds}
                setOpenChat={setOpenChat}
              />
            </div>
          </FullScreenModal>
        </Portal>
      </Show>
      <div class="mt-12 flex w-full flex-col items-center space-y-4">
        <Switch>
          <Match
            when={
              props.loading() ||
              (resultChunks().length === 0 && !clientSideRequestFinished())
            }
          >
            <div
              class="text-primary inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-current border-magenta border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]"
              role="status"
            >
              <span class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">
                Loading...
              </span>
            </div>
          </Match>
          <Match when={noResults()}>
            <div class="text-2xl">No results found</div>
          </Match>
          <Match when={!props.loading() && groupResultChunks().length == 0}>
            <div class="flex w-full max-w-7xl flex-col space-y-4 px-1 min-[360px]:px-4 sm:px-8 md:px-20">
              <For each={resultChunks()}>
                {(chunk) => (
                  <div>
                    <ScoreChunkArray
                      totalGroupPages={totalCollectionPages()}
                      chunkGroups={chunkCollections()}
                      chunks={chunk.metadata}
                      score={chunk.score}
                      bookmarks={bookmarks()}
                      setOnDelete={setOnDelete}
                      setShowConfirmModal={setShowConfirmDeleteModal}
                      showExpand={clientSideRequestFinished()}
                      setChunkGroups={setChunkCollections}
                      setSelectedIds={setSelectedIds}
                      selectedIds={selectedIds}
                    />
                  </div>
                )}
              </For>
            </div>
            <Show when={resultChunks().length > 0}>
              <div class="mx-auto my-12 flex items-center space-x-2">
                <PaginationController page={props.page} totalPages={100} />
              </div>
            </Show>
            <div>
              <div
                data-dial-init
                class="group fixed bottom-6 right-6"
                onMouseEnter={() => {
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.remove("hidden");
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.add("flex");
                }}
                onMouseLeave={() => {
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.add("hidden");
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.remove("flex");
                }}
              >
                <div
                  id="speed-dial-menu-text-outside-button"
                  class="mb-4 hidden flex-col items-center space-y-2"
                >
                  <button
                    type="button"
                    class="relative h-[52px] w-[52px] items-center justify-center rounded-lg border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:ring-4 focus:ring-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-400"
                    onClick={() => {
                      setSelectedIds(
                        resultChunks()
                          .flatMap((c) => {
                            return c.metadata.map((m) => m.id);
                          })
                          .slice(0, 10),
                      );
                      setOpenChat(true);
                    }}
                  >
                    <IoDocumentsOutline class="mx-auto h-7 w-7" />
                    <span class="font-sm absolute -left-[8.5rem] top-1/2 mb-px block -translate-y-1/2 break-words bg-white/30 text-sm backdrop-blur-xl dark:bg-black/30">
                      Chat with all results
                    </span>
                  </button>
                  <button
                    type="button"
                    class="relative h-[52px] w-[52px] items-center justify-center rounded-lg border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:ring-4 focus:ring-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-400"
                    onClick={() => {
                      setOpenChat(true);
                    }}
                  >
                    <IoDocumentOutline class="mx-auto h-7 w-7" />
                    <span class="font-sm absolute -left-[10.85rem] top-1/2 mb-px block -translate-y-1/2 bg-white/30 text-sm backdrop-blur-xl dark:bg-black/30">
                      Chat with selected results
                    </span>
                  </button>
                </div>
                <button
                  type="button"
                  class="flex h-14 w-14 items-center justify-center rounded-lg bg-magenta-500 text-white hover:bg-magenta-400 focus:outline-none focus:ring-4 focus:ring-magenta-300 dark:bg-magenta-500 dark:hover:bg-magenta-400 dark:focus:ring-magenta-600"
                >
                  <AiOutlineRobot class="h-7 w-7 fill-current text-white" />
                  <span class="sr-only">Open actions menu</span>
                </button>
              </div>
            </div>
          </Match>
          <Match when={!props.loading() && groupResultChunks().length > 0}>
            <For each={groupResultChunks()}>
              {(group) => {
                const [groupExpanded, setGroupExpanded] = createSignal(true);

                const toggle = () => {
                  setGroupExpanded(!groupExpanded());
                };

                return (
                  <div class="flex w-full max-w-7xl flex-col space-y-4 px-1 min-[360px]:px-4 sm:px-8 md:px-20">
                    <div
                      onClick={toggle}
                      classList={{
                        "mx-8 flex items-center space-x-2 rounded bg-neutral-100 px-4 py-4 dark:bg-neutral-800":
                          true,
                        "-mb-2": groupExpanded(),
                      }}
                    >
                      <Show when={groupExpanded()}>
                        <FaSolidChevronUp />
                      </Show>
                      <Show when={!groupExpanded()}>
                        <FaSolidChevronDown />
                      </Show>
                      <div>{group.group_name}</div>
                    </div>
                    <Show when={groupExpanded()}>
                      <For each={group.metadata}>
                        {(chunk) => (
                          <div class="ml-5 flex space-y-4">
                            <ScoreChunkArray
                              totalGroupPages={totalCollectionPages()}
                              chunkGroups={chunkCollections()}
                              chunks={chunk.metadata}
                              score={chunk.score}
                              bookmarks={bookmarks()}
                              setOnDelete={setOnDelete}
                              setShowConfirmModal={setShowConfirmDeleteModal}
                              showExpand={clientSideRequestFinished()}
                              setChunkGroups={setChunkCollections}
                              setSelectedIds={setSelectedIds}
                              selectedIds={selectedIds}
                            />
                          </div>
                        )}
                      </For>
                    </Show>
                  </div>
                );
              }}
            </For>
            <Show when={resultChunks().length > 0}>
              <div class="mx-auto my-12 flex items-center space-x-2">
                <PaginationController page={props.page} totalPages={100} />
              </div>
            </Show>
            <div>
              <div
                data-dial-init
                class="group fixed bottom-6 right-6"
                onMouseEnter={() => {
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.remove("hidden");
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.add("flex");
                }}
                onMouseLeave={() => {
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.add("hidden");
                  document
                    .getElementById("speed-dial-menu-text-outside-button")
                    ?.classList.remove("flex");
                }}
              >
                <div
                  id="speed-dial-menu-text-outside-button"
                  class="mb-4 hidden flex-col items-center space-y-2"
                >
                  <button
                    type="button"
                    class="relative h-[52px] w-[52px] items-center justify-center rounded-lg border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:ring-4 focus:ring-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-400"
                    onClick={() => {
                      setSelectedIds(
                        resultChunks()
                          .flatMap((c) => {
                            return c.metadata.map((m) => m.id);
                          })
                          .slice(0, 10),
                      );
                      setOpenChat(true);
                    }}
                  >
                    <IoDocumentsOutline class="mx-auto h-7 w-7" />
                    <span class="font-sm absolute -left-[8.5rem] top-1/2 mb-px block -translate-y-1/2 break-words bg-white/30 text-sm backdrop-blur-xl dark:bg-black/30">
                      Chat with all results
                    </span>
                  </button>
                  <button
                    type="button"
                    class="relative h-[52px] w-[52px] items-center justify-center rounded-lg border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:ring-4 focus:ring-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-400"
                    onClick={() => {
                      setOpenChat(true);
                    }}
                  >
                    <IoDocumentOutline class="mx-auto h-7 w-7" />
                    <span class="font-sm absolute -left-[10.85rem] top-1/2 mb-px block -translate-y-1/2 bg-white/30 text-sm backdrop-blur-xl dark:bg-black/30">
                      Chat with selected results
                    </span>
                  </button>
                </div>
                <button
                  type="button"
                  class="flex h-14 w-14 items-center justify-center rounded-lg bg-magenta-500 text-white hover:bg-magenta-400 focus:outline-none focus:ring-4 focus:ring-magenta-300 dark:bg-magenta-500 dark:hover:bg-magenta-400 dark:focus:ring-magenta-600"
                >
                  <AiOutlineRobot class="h-7 w-7 fill-current text-white" />
                  <span class="sr-only">Open actions menu</span>
                </button>
              </div>
            </div>
          </Match>
        </Switch>
      </div>
      <ConfirmModal
        showConfirmModal={showConfirmDeleteModal}
        setShowConfirmModal={setShowConfirmDeleteModal}
        onConfirm={onDelete}
        message="Are you sure you want to delete this chunk?"
      />
    </>
  );
};

export default ResultsPage;
