import {
  Show,
  createEffect,
  createSignal,
  For,
  onMount,
  onCleanup,
} from "solid-js";
import {
  type ChunkCollectionDTO,
  type ChunkCollectionBookmarkDTO,
  ChunkBookmarksDTO,
  BookmarkDTO,
  ScoreChunkDTO,
  ChunkCollectionSearchDTO,
  isScoreChunkDTO,
  isChunkCollectionPageDTO,
  ChunkMetadata,
} from "../../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { BiRegularLogInCircle, BiRegularXCircle } from "solid-icons/bi";
import { FiEdit, FiTrash } from "solid-icons/fi";
import { ConfirmModal } from "./Atoms/ConfirmModal";
import { PaginationController } from "./Atoms/PaginationController";
import { ScoreChunkArray } from "./ScoreChunkArray";
import SearchForm from "./SearchForm";
import type { Filters } from "./ResultsPage";
import ChunkMetadataDisplay from "./ChunkMetadataDisplay";
import { Portal } from "solid-js/web";
import ChatPopup from "./ChatPopup";
import { AiOutlineRobot } from "solid-icons/ai";
import { IoDocumentOutline, IoDocumentsOutline } from "solid-icons/io";
import { currentUser } from "../stores/userStore";
import { useStore } from "@nanostores/solid";
import { currentDataset } from "../stores/datasetStore";

export interface CollectionPageProps {
  collectionID: string;
  defaultCollectionChunks: {
    metadata: ChunkCollectionBookmarkDTO | ChunkCollectionSearchDTO;
    status: number;
  };
  page: number;
  query: string;
  searchType: string;
  filters: Filters;
}

export const CollectionPage = (props: CollectionPageProps) => {
  const apiHost: string = import.meta.env.PUBLIC_API_HOST as string;
  const $dataset = useStore(currentDataset);

  const chunkMetadatasWithVotes: BookmarkDTO[] = [];
  const searchChunkMetadatasWithVotes: ScoreChunkDTO[] = [];

  // Sometimes this will error server-side so we have to handle it
  try {
    if (
      props.defaultCollectionChunks.metadata.bookmarks.length > 0 &&
      !isScoreChunkDTO(props.defaultCollectionChunks.metadata.bookmarks[0])
    ) {
      chunkMetadatasWithVotes.push(
        ...(props.defaultCollectionChunks.metadata.bookmarks as BookmarkDTO[]),
      );
    } else if (
      props.defaultCollectionChunks.metadata.bookmarks.length > 0 &&
      isScoreChunkDTO(props.defaultCollectionChunks.metadata.bookmarks[0])
    ) {
      searchChunkMetadatasWithVotes.push(
        ...(props.defaultCollectionChunks.metadata
          .bookmarks as ScoreChunkDTO[]),
      );
    }
  } catch (e) {
    console.error(e);
  }

  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [metadatasWithVotes, setMetadatasWithVotes] = createSignal<
    BookmarkDTO[]
  >(chunkMetadatasWithVotes);
  const [searchMetadatasWithVotes, setSearchMetadatasWithVotes] = createSignal<
    ScoreChunkDTO[]
  >(searchChunkMetadatasWithVotes);
  const [clientSideRequestFinished, setClientSideRequestFinished] =
    createSignal(false);
  const [collectionInfo, setCollectionInfo] = createSignal<ChunkCollectionDTO>(
    props.defaultCollectionChunks.metadata.collection,
  );
  const [chunkCollections, setChunkCollections] = createSignal<
    ChunkCollectionDTO[]
  >([]);
  const [bookmarks, setBookmarks] = createSignal<ChunkBookmarksDTO[]>([]);
  const [error, setError] = createSignal("");
  const [fetchingCollections, setFetchingCollections] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [editing, setEditing] = createSignal(false);
  const $currentUser = useStore(currentUser);
  const [totalPages, setTotalPages] = createSignal(
    props.defaultCollectionChunks.metadata.total_pages,
  );
  const [loadingRecommendations, setLoadingRecommendations] =
    createSignal(false);
  const [recommendedChunks, setRecommendedChunks] = createSignal<
    ChunkMetadata[]
  >([]);

  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);

  const [
    showConfirmCollectionDeleteModal,
    setShowConfirmCollectionmDeleteModal,
  ] = createSignal(false);

  const [totalCollectionPages, setTotalCollectionPages] = createSignal(1);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onCollectionDelete, setOnCollectionDelete] = createSignal(() => {});

  const [openChat, setOpenChat] = createSignal(false);
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);

  onMount(() => {
    fetchBookmarks();
  });

  createEffect(() => {
    const resultsLength = metadatasWithVotes().length;
    if (!openChat()) {
      setSelectedIds((prev) => (prev.length < resultsLength ? prev : []));
    }
  });

  createEffect(() => {
    const abortController = new AbortController();
    let collection_id: string | null = null;
    const currentDataset = $dataset();
    if (!currentDataset) return;
    if (props.query === "") {
      void fetch(
        `${apiHost}/chunk_collection/${props.collectionID}/${props.page}`,
        {
          method: "GET",
          credentials: "include",
          signal: abortController.signal,
          headers: {
            "AF-Dataset": currentDataset.dataset.id,
          },
        },
      ).then((response) => {
        if (response.ok) {
          void response.json().then((data) => {
            const collectionBookmarks = data as ChunkCollectionBookmarkDTO;
            collection_id = collectionBookmarks.collection.id;
            setCollectionInfo(collectionBookmarks.collection);
            setTotalPages(collectionBookmarks.total_pages);
            setMetadatasWithVotes(collectionBookmarks.bookmarks);
            setError("");
          });
        }
        if (response.status == 403) {
          setError("You are not authorized to view this theme");
        }
        if (response.status == 404) {
          setError("Theme not found, it never existed or was deleted");
        }
        if (response.status == 401) {
          setError("You must be logged in and authorized to view this theme");
          setShowNeedLoginModal(true);
        }
        setClientSideRequestFinished(true);
      });
    } else {
      void fetch(`${apiHost}/chunk_collection/search/${props.page}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "AF-Dataset": currentDataset.dataset.id,
        },
        signal: abortController.signal,
        credentials: "include",
        body: JSON.stringify({
          content: props.query,
          tag_set: props.filters.tagSet,
          link: props.filters.link,
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          filters: props.filters.metadataFilters,
          collection_id: props.collectionID,
          search_type: props.searchType,
        }),
      }).then((response) => {
        if (response.ok) {
          void response.json().then((data) => {
            const collectionBookmarks = data as ChunkCollectionSearchDTO;
            collection_id = collectionBookmarks.collection.id;
            setCollectionInfo(collectionBookmarks.collection);
            setTotalPages(collectionBookmarks.total_pages);
            setSearchMetadatasWithVotes(collectionBookmarks.bookmarks);
            setError("");
          });
        }
        if (response.status == 403) {
          setError("You are not authorized to view this theme");
        }
        if (response.status == 401) {
          setShowNeedLoginModal(true);
        }
        setClientSideRequestFinished(true);
      });

      onCleanup(() => {
        abortController.abort();
      });
    }

    fetchChunkCollections();

    setOnCollectionDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/chunk_collection`, {
          method: "DELETE",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
            "AF-Dataset": currentDataset.dataset.id,
          },
          signal: abortController.signal,
          body: JSON.stringify({
            collection_id: collection_id,
          }),
        }).then((response) => {
          setDeleting(false);
          if (response.ok) {
            window.location.href = "/";
          }
          if (response.status == 403) {
            setDeleting(false);
          }
          if (response.status == 401) {
            setShowNeedLoginModal(true);
          }
        });
      };
    });
  });

  createEffect(() => {
    resizeTextarea(
      document.getElementById(
        "collection-query-textarea",
      ) as HTMLTextAreaElement | null,
    );
  });

  // Fetch the chunk collections for the auth'ed user
  const fetchChunkCollections = () => {
    const currentDataset = $dataset();
    if (!currentDataset) return;
    if (!$currentUser()) return;

    void fetch(`${apiHost}/chunk_collection/1`, {
      method: "GET",
      credentials: "include",
      headers: {
        "AF-Dataset": currentDataset.dataset.id,
      },
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isChunkCollectionPageDTO(data)) {
            setChunkCollections(data.collections);
            setTotalCollectionPages(data.total_pages);
          }
        });
      }
    });
  };

  const fetchBookmarks = () => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    void fetch(`${apiHost}/chunk_collection/bookmark`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": currentDataset.dataset.id,
      },
      body: JSON.stringify({
        chunk_ids: metadatasWithVotes().flatMap((m) => {
          return m.metadata.map((c) => c.id);
        }),
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setBookmarks(data as ChunkBookmarksDTO[]);
        });
      }
    });
  };

  const updateCollection = () => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    setFetchingCollections(true);
    const body = {
      collection_id: collectionInfo().id,
      name: collectionInfo().name,
      description: collectionInfo().description,
    };
    void fetch(`${apiHost}/chunk_collection`, {
      method: "PUT",
      credentials: "include",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": currentDataset.dataset.id,
      },
    }).then((response) => {
      setFetchingCollections(false);
      if (response.ok) {
        setEditing(false);
      }
      if (response.status == 401) {
        setShowNeedLoginModal(true);
      }
    });
  };

  const fetchRecommendations = (
    ids: string[],
    prev_recommendations: ChunkMetadata[],
  ) => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    setLoadingRecommendations(true);
    void fetch(`${apiHost}/chunk/recommend`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": currentDataset.dataset.id,
      },
      body: JSON.stringify({
        positive_chunk_ids: ids,
        limit: prev_recommendations.length + 10,
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const typed_data = data as ChunkMetadata[];
          const deduped_data = typed_data.filter((d) => {
            return !prev_recommendations.some((c) => c.id == d.id);
          });
          const new_recommendations = [
            ...prev_recommendations,
            ...deduped_data,
          ];
          setLoadingRecommendations(false);
          setRecommendedChunks(new_recommendations);
        });
      }
      if (response.status == 401) {
        setShowNeedLoginModal(true);
        setLoadingRecommendations(false);
      }
    });
  };

  const resizeTextarea = (textarea: HTMLTextAreaElement | null) => {
    if (!textarea) return;

    textarea.style.height = `${textarea.scrollHeight}px`;
  };

  return (
    <>
      <Show when={openChat()}>
        <Portal>
          <FullScreenModal isOpen={openChat} setIsOpen={setOpenChat}>
            <div class="max-h-[75vh] min-h-[75vh] min-w-[75vw] max-w-[75vw] overflow-y-auto rounded-md scrollbar-thin">
              <ChatPopup
                chunks={() =>
                  metadatasWithVotes() as unknown as ScoreChunkDTO[]
                }
                selectedIds={selectedIds}
                setShowNeedLoginModal={setShowNeedLoginModal}
                setOpenChat={setOpenChat}
              />
            </div>
          </FullScreenModal>
        </Portal>
      </Show>
      <div class="flex w-full flex-col items-center space-y-2">
        <Show when={error().length == 0}>
          <div class="flex w-full max-w-6xl items-center justify-end space-x-2 px-4 sm:px-8 md:px-20">
            <Show
              when={chunkCollections().some(
                (collection) => collection.id == collectionInfo().id,
              )}
            >
              <button
                classList={{
                  "h-fit text-red-700 dark:text-red-400": true,
                  "animate-pulse": deleting(),
                }}
                onClick={() => setShowConfirmCollectionmDeleteModal(true)}
              >
                <FiTrash class="h-5 w-5" />
              </button>
              <button onClick={() => setEditing((prev) => !prev)}>
                <FiEdit class="h-5 w-5" />
              </button>
            </Show>
          </div>
          <Show when={!editing()}>
            <div class="flex w-full items-center justify-center">
              <h1 class="break-all text-center text-lg min-[320px]:text-xl sm:text-3xl">
                {collectionInfo().name}
              </h1>
            </div>
            <Show when={collectionInfo().description.length > 0 && !editing()}>
              <div class="mx-auto flex max-w-[300px] justify-items-center gap-x-2 md:max-w-fit">
                <div class="text-center text-lg font-semibold">
                  Description:
                </div>
                <div class="line-clamp-1 flex w-full justify-start text-center text-lg">
                  {collectionInfo().description}
                </div>
              </div>
            </Show>
          </Show>

          <Show when={editing()}>
            <div class="vertical-align-left mt-8 grid w-full max-w-6xl auto-rows-max grid-cols-[1fr,3fr] gap-y-2 px-4 sm:px-8 md:px-20">
              <h1 class="text-md min-[320px]:text-md sm:text-md mt-10 text-left font-bold">
                Name:
              </h1>
              <input
                type="text"
                class="mt-10 max-h-fit w-full rounded-md bg-neutral-200 px-2 py-1 dark:bg-neutral-700"
                value={collectionInfo().name}
                onInput={(e) => {
                  setCollectionInfo({
                    ...collectionInfo(),
                    name: e.target.value,
                  });
                }}
              />
              <div class="text-md mr-2 font-semibold">Description:</div>
              <textarea
                class="w-full justify-start rounded-md bg-neutral-200 px-2 py-1 dark:bg-neutral-700"
                value={collectionInfo().description}
                onInput={(e) => {
                  setCollectionInfo({
                    ...collectionInfo(),
                    description: e.target.value,
                  });
                }}
              />
            </div>
            <div class="mt-4 flex w-full max-w-6xl justify-end px-4 sm:px-8 md:px-20">
              <button
                classList={{
                  "!pointer-events-auto relative max-h-10 mt-2 mr-2 items-end justify-end rounded-md p-2 text-center bg-red-500":
                    true,
                  "animate-pulse": fetchingCollections(),
                }}
                onClick={() => setEditing(false)}
              >
                Cancel
              </button>
              <button
                classList={{
                  "!pointer-events-auto relative max-h-10 mt-2 mr-2 items-end justify-end rounded-md p-2 text-center bg-green-500":
                    true,
                  "animate-pulse": fetchingCollections(),
                }}
                onClick={() => updateCollection()}
              >
                Save
              </button>
            </div>
          </Show>
        </Show>
        <div class="flex w-full max-w-6xl flex-col space-y-4 border-t border-neutral-500 px-4 sm:px-8 md:px-20">
          <Show when={props.query != ""}>
            <button
              class="relative mx-auto ml-8 mt-8 h-fit max-h-[240px] rounded-md bg-neutral-100 p-2 dark:bg-neutral-700"
              onClick={() =>
                (window.location.href = `/collection/${props.collectionID}`)
              }
            >
              ← Back
            </button>
          </Show>
          <Show when={metadatasWithVotes().length > 0}>
            <div class="mx-auto w-full max-w-6xl">
              <div
                classList={{
                  "mx-auto w-full max-w-[calc(100%-32px)] min-[360px]:max-w-[calc(100%-64px)]":
                    true,
                  "mt-8": props.query == "",
                }}
              >
                <SearchForm
                  query={props.query}
                  filters={props.filters}
                  searchType={props.searchType}
                  collectionID={props.collectionID}
                />
              </div>
            </div>
          </Show>
          <Show when={props.query != ""}>
            <div class="flex w-full flex-col items-center rounded-md p-2">
              <div class="text-xl font-semibold">
                Search results for "{props.query}"
              </div>
            </div>
          </Show>
          <For
            each={
              props.query == ""
                ? metadatasWithVotes()
                : searchMetadatasWithVotes()
            }
          >
            {(chunk) => (
              <div class="mt-4">
                <ScoreChunkArray
                  totalCollectionPages={totalCollectionPages()}
                  chunks={chunk.metadata}
                  score={isScoreChunkDTO(chunk) ? chunk.score : 0}
                  collection={true}
                  setShowModal={setShowNeedLoginModal}
                  chunkCollections={chunkCollections()}
                  bookmarks={bookmarks()}
                  setOnDelete={setOnDelete}
                  setShowConfirmModal={setShowConfirmDeleteModal}
                  showExpand={clientSideRequestFinished()}
                  setChunkCollections={setChunkCollections}
                  setSelectedIds={setSelectedIds}
                  selectedIds={selectedIds}
                />
              </div>
            )}
          </For>
          <div class="mx-auto my-12 flex items-center justify-center space-x-2">
            <PaginationController page={props.page} totalPages={totalPages()} />
          </div>
          <Show when={recommendedChunks().length > 0}>
            <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] min-[360px]:max-w-[calc(100%-64px)]">
              <div class="flex w-full flex-col items-center rounded-md p-2">
                <div class="text-xl font-semibold">Related Chunks</div>
              </div>
              <For each={recommendedChunks()}>
                {(chunk) => (
                  <>
                    <div class="mt-4">
                      <ChunkMetadataDisplay
                        totalCollectionPages={totalCollectionPages()}
                        chunk={chunk}
                        chunkCollections={chunkCollections()}
                        bookmarks={bookmarks()}
                        setShowModal={setShowNeedLoginModal}
                        setShowConfirmModal={setShowConfirmDeleteModal}
                        fetchChunkCollections={fetchChunkCollections}
                        setChunkCollections={setChunkCollections}
                        setOnDelete={setOnDelete}
                        showExpand={true}
                      />
                    </div>
                  </>
                )}
              </For>
            </div>
          </Show>
          <Show when={metadatasWithVotes().length > 0}>
            <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] min-[360px]:max-w-[calc(100%-64px)]">
              <button
                classList={{
                  "w-full rounded  bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800":
                    true,
                  "animate-pulse": loadingRecommendations(),
                }}
                onClick={() =>
                  fetchRecommendations(
                    metadatasWithVotes().map(
                      (m) => m.metadata[0].qdrant_point_id,
                    ),
                    recommendedChunks(),
                  )
                }
              >
                {recommendedChunks().length == 0 ? "Get" : "Get More"} Related
                Chunks
              </button>
            </div>
          </Show>
          <Show when={error().length > 0}>
            <div class="flex w-full flex-col items-center rounded-md p-2">
              <div class="text-xl font-semibold text-red-500">{error()}</div>
            </div>
          </Show>
          <Show
            when={
              metadatasWithVotes().length == 0 &&
              searchMetadatasWithVotes().length == 0 &&
              clientSideRequestFinished()
            }
          >
            <div class="flex w-full flex-col items-center rounded-md p-2">
              <div class="text-xl font-semibold">
                This theme is currently empty
              </div>
            </div>
          </Show>
        </div>
      </div>
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
                const searchResults = searchMetadatasWithVotes();
                if (searchResults.length > 0) {
                  setSelectedIds(
                    searchResults
                      .flatMap((c) => {
                        return c.metadata.map((m) => m.id);
                      })
                      .slice(0, 10),
                  );
                  setOpenChat(true);
                } else {
                  setSelectedIds(
                    metadatasWithVotes()
                      .flatMap((c) => {
                        return c.metadata.map((m) => m.id);
                      })
                      .slice(0, 10),
                  );
                }
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
            <AiOutlineRobot class="h-7 w-7" />
            <span class="sr-only">Open actions menu</span>
          </button>
        </div>
      </div>
      <Show when={showNeedLoginModal()}>
        <FullScreenModal
          isOpen={showNeedLoginModal}
          setIsOpen={setShowNeedLoginModal}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current  !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold">
              Login or register to bookmark chunks, vote, get recommend chunks
              or manage your collections
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href={`${apiHost}/auth?dataset_id=${
                  $dataset()?.dataset.name ?? ""
                }`}
              >
                Login/Register
                <BiRegularLogInCircle class="h-6 w-6  fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
      <ConfirmModal
        showConfirmModal={showConfirmDeleteModal}
        setShowConfirmModal={setShowConfirmDeleteModal}
        onConfirm={onDelete}
        message="Are you sure you want to delete this chunk?"
      />
      <ConfirmModal
        showConfirmModal={showConfirmCollectionDeleteModal}
        setShowConfirmModal={setShowConfirmCollectionmDeleteModal}
        onConfirm={onCollectionDelete}
        message="Are you sure you want to delete this theme?"
      />
    </>
  );
};
