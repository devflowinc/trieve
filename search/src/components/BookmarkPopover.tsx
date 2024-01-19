import { For, Setter, Show, createEffect, createSignal } from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { RiSystemAddFill } from "solid-icons/ri";
import {
  isChunkCollectionPageDTO,
  type ChunkBookmarksDTO,
  type ChunkCollectionDTO,
  type ChunkMetadata,
} from "../../utils/apiTypes";
import InputRowsForm from "./Atoms/InputRowsForm";
import { VsBookmark } from "solid-icons/vs";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { useStore } from "@nanostores/solid";
import { currentUser } from "../stores/userStore";
import { currentDataset } from "../stores/datasetStore";

export interface BookmarkPopoverProps {
  chunkMetadata: ChunkMetadata;
  chunkCollections: ChunkCollectionDTO[];
  totalCollectionPages: number;
  setLoginModal?: Setter<boolean>;
  bookmarks: ChunkBookmarksDTO[];
  setChunkCollections?: Setter<ChunkCollectionDTO[]>;
}

const BookmarkPopover = (props: BookmarkPopoverProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const $dataset = useStore(currentDataset);
  const $currentUser = useStore(currentUser);

  const [refetchingChunkCollections, setRefetchingChunkCollections] =
    createSignal(false);
  const [refetchingBookmarks, setRefetchingBookmarks] = createSignal(false);
  const [showCollectionForm, setShowCollectionForm] = createSignal(false);
  const [notLoggedIn, setNotLoggedIn] = createSignal(false);
  const [collectionFormTitle, setCollectionFormTitle] = createSignal("");
  const [usingPanel, setUsingPanel] = createSignal(false);
  const [bookmarks, setBookmarks] = createSignal<ChunkBookmarksDTO[]>([]);
  const [localCollectionPage, setLocalCollectionPage] = createSignal(1);
  const [localChunkCollections, setLocalChunkCollections] = createSignal<
    ChunkCollectionDTO[]
  >([]);

  createEffect(() => {
    const collectionsToAdd: ChunkCollectionDTO[] = [];
    props.bookmarks.forEach((b) => {
      b.slim_collections.forEach((c) => {
        c.of_current_user &&
          collectionsToAdd.push({
            id: c.id,
            name: c.name,
            description: "",
            author_id: c.author_id,
            created_at: "",
            updated_at: "",
          });
      });
    });

    setBookmarks(props.bookmarks);
    setLocalChunkCollections([...collectionsToAdd, ...props.chunkCollections]);
  });

  createEffect((prevPage) => {
    const curPage = localCollectionPage();
    if (curPage == prevPage) {
      return curPage;
    }

    const chunkBookmarks = bookmarks();
    const setChunkCollections = props.setChunkCollections;
    refetchCollections(curPage, chunkBookmarks, setChunkCollections);

    return curPage;
  }, 1);

  createEffect(() => {
    if ($currentUser()?.id === undefined) {
      return;
    }
    if (!refetchingChunkCollections()) {
      return;
    }

    const curPage = localCollectionPage();
    const chunkBookmarks = bookmarks();
    const setChunkCollections = props.setChunkCollections;
    refetchCollections(curPage, chunkBookmarks, setChunkCollections);
    setRefetchingChunkCollections(false);
  });

  createEffect(() => {
    if ($currentUser()?.id === undefined) {
      return;
    }
    if (!refetchingBookmarks()) {
      return;
    }

    const curCollectionPage = localCollectionPage();
    refetchBookmarks(curCollectionPage);
    setRefetchingBookmarks(false);
  });

  const refetchCollections = (
    curPage: number,
    chunkBookmarks: ChunkBookmarksDTO[],
    setChunkCollections: Setter<ChunkCollectionDTO[]> | undefined,
  ) => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    void fetch(`${apiHost}/chunk_collection/${localCollectionPage()}`, {
      method: "GET",
      headers: {
        "TR-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
    }).then((response) => {
      if (!setChunkCollections) return;

      if (response.ok) {
        void response.json().then((data) => {
          if (isChunkCollectionPageDTO(data)) {
            if (curPage !== 1) {
              setLocalChunkCollections(data.collections);
              return;
            }

            const collectionsToAdd: ChunkCollectionDTO[] = [];

            chunkBookmarks.forEach((chunkBookmark) => {
              chunkBookmark.slim_collections.forEach((collection) => {
                if (collection.of_current_user) {
                  const chunkCollection: ChunkCollectionDTO = {
                    id: collection.id,
                    name: collection.name,
                    description: "",
                    author_id: collection.author_id,
                    created_at: "",
                    updated_at: "",
                  };

                  collectionsToAdd.push(chunkCollection);
                }
              });
            });

            const deDupedPrev = data.collections.filter((collection) => {
              return (
                collectionsToAdd.find(
                  (collectionToAdd) => collectionToAdd.id == collection.id,
                ) == undefined
              );
            });

            const updatedCollections = [...collectionsToAdd, ...deDupedPrev];
            setLocalChunkCollections(updatedCollections);
            setChunkCollections(updatedCollections);
          }
        });
      }

      if (response.status == 401) {
        setNotLoggedIn(true);
      }
    });
  };

  const refetchBookmarks = (curPage: number) => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    void fetch(`${apiHost}/chunk_collection/bookmark`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": currentDataset.dataset.id,
      },
      body: JSON.stringify({
        chunk_ids: [props.chunkMetadata.id],
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const chunkBookmarks = data as ChunkBookmarksDTO[];

          setBookmarks(data as ChunkBookmarksDTO[]);

          if (curPage !== 1) {
            return;
          }

          const collectionsToAdd: ChunkCollectionDTO[] = [];

          chunkBookmarks.forEach((chunkBookmark) => {
            chunkBookmark.slim_collections.forEach((collection) => {
              if (collection.of_current_user) {
                const chunkCollection: ChunkCollectionDTO = {
                  id: collection.id,
                  name: collection.name,
                  description: "",
                  author_id: collection.author_id,
                  created_at: "",
                  updated_at: "",
                };

                collectionsToAdd.push(chunkCollection);
              }
            });
          });

          setLocalChunkCollections((prev) => {
            const deDupedPrev = prev.filter((collection) => {
              return (
                collectionsToAdd.find(
                  (collectionToAdd) => collectionToAdd.id == collection.id,
                ) == undefined
              );
            });

            return [...collectionsToAdd, ...deDupedPrev];
          });
        });
      }
    });
  };

  return (
    <Popover defaultOpen={false} class="relative">
      {({ isOpen, setState }) => (
        <div>
          <div class="flex items-center">
            <PopoverButton
              title="Bookmark"
              onClick={() => {
                if (notLoggedIn() || $currentUser()?.id === undefined) {
                  props.setLoginModal?.(true);
                  return;
                }
                refetchBookmarks(localCollectionPage());
              }}
            >
              <VsBookmark class="z-0 h-5 w-5 fill-current" />
            </PopoverButton>
          </div>
          <Show
            when={
              (isOpen() || usingPanel()) &&
              !notLoggedIn() &&
              !($currentUser()?.id === undefined)
            }
          >
            <PopoverPanel
              unmount={false}
              class="absolute z-50 w-screen max-w-xs -translate-x-[300px] translate-y-1"
              onMouseEnter={() => setUsingPanel(true)}
              onMouseLeave={() => setUsingPanel(false)}
              onClick={() => setState(true)}
            >
              <Menu class=" flex w-full flex-col justify-end space-y-2 overflow-hidden rounded bg-white py-4 shadow-2xl dark:bg-shark-700">
                <div class="mb-3 w-full px-4 text-center text-lg font-bold">
                  Manage Themes For This Chunk
                </div>
                <MenuItem as="button" aria-label="Empty" />
                <div class="max-w-screen mx-1 max-h-[20vh] transform justify-end space-y-2 overflow-y-auto rounded px-4 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-600 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-400">
                  <For each={localChunkCollections()}>
                    {(collection, idx) => {
                      return (
                        <>
                          <Show when={idx() != 0}>
                            <div class="h-px w-full bg-neutral-200 dark:bg-neutral-700" />
                          </Show>
                          <div class="flex w-full items-center justify-between space-x-2">
                            <a
                              href={`/collection/${collection.id}`}
                              class="max-w-[80%] underline"
                            >
                              {collection.name}
                            </a>

                            <input
                              type="checkbox"
                              checked={
                                bookmarks().find((bookmark) =>
                                  bookmark.slim_collections
                                    .map((slimCollection) => slimCollection.id)
                                    .includes(collection.id),
                                )
                                  ? true
                                  : false
                              }
                              onChange={(e) => {
                                const currentDataset = $dataset();
                                if (!currentDataset) return;
                                void fetch(
                                  `${apiHost}/chunk_collection/${collection.id}`,
                                  {
                                    method: e.currentTarget.checked
                                      ? "POST"
                                      : "DELETE",
                                    headers: {
                                      "Content-Type": "application/json",
                                      "TR-Dataset": currentDataset.dataset.id,
                                    },
                                    credentials: "include",
                                    body: JSON.stringify({
                                      chunk_metadata_id: props.chunkMetadata.id,
                                    }),
                                  },
                                ).then((response) => {
                                  if (!response.ok) {
                                    e.currentTarget.checked =
                                      !e.currentTarget.checked;
                                  }
                                  setRefetchingBookmarks(true);
                                });
                                setState(true);
                              }}
                              class="h-4 w-4 cursor-pointer rounded-sm border-gray-300 bg-neutral-500 accent-turquoise focus:ring-neutral-200 dark:border-neutral-700 dark:focus:ring-neutral-600"
                            />
                          </div>
                        </>
                      );
                    }}
                  </For>
                  <div class="flex items-center justify-between">
                    <div />
                    <div class="flex items-center">
                      <div class="text-sm text-neutral-400">
                        {localCollectionPage()} /{" "}
                        {props.totalCollectionPages == 0
                          ? 1
                          : props.totalCollectionPages}
                      </div>
                      <button
                        class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                        disabled={localCollectionPage() == 1}
                        onClick={() => {
                          setState(true);
                          setLocalCollectionPage((prev) => prev - 1);
                        }}
                      >
                        <BiRegularChevronLeft class="h-6 w-6 fill-current" />
                      </button>
                      <button
                        class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                        disabled={
                          localCollectionPage() ==
                          (props.totalCollectionPages == 0
                            ? 1
                            : props.totalCollectionPages)
                        }
                        onClick={() => {
                          setState(true);
                          setLocalCollectionPage((prev) => prev + 1);
                        }}
                      >
                        <BiRegularChevronRight class="h-6 w-6 fill-current" />
                      </button>
                    </div>
                  </div>
                </div>
                <Show when={showCollectionForm()}>
                  <div class="mx-4 rounded bg-gray-100 py-2 dark:bg-neutral-800">
                    <div class="px-2 text-lg font-bold">
                      Create New Collection
                    </div>
                    <div>
                      <InputRowsForm
                        createButtonText="Create collection"
                        onCreate={() => {
                          const title = collectionFormTitle();
                          if (title.trim() == "") return;
                          const currentDataset = $dataset();
                          if (!currentDataset) return;
                          void fetch(`${apiHost}/chunk_collection`, {
                            method: "POST",
                            headers: {
                              "Content-Type": "application/json",
                              "TR-Dataset": currentDataset.dataset.id,
                            },
                            credentials: "include",
                            body: JSON.stringify({
                              name: title,
                              description: "",
                            }),
                          }).then(() => {
                            setRefetchingChunkCollections(true);
                            setShowCollectionForm(false);
                            setCollectionFormTitle("");
                            setState(true);
                          });
                        }}
                        onCancel={() => {
                          setShowCollectionForm(false);
                          setState(true);
                        }}
                        inputGroups={[
                          {
                            label: "Title",
                            inputValue: collectionFormTitle,
                            setInputValue: setCollectionFormTitle,
                          },
                        ]}
                      />
                    </div>
                  </div>
                </Show>
                {!showCollectionForm() && (
                  <div class="px-4 pt-4">
                    <MenuItem
                      as="button"
                      onClick={() => {
                        setShowCollectionForm(true);
                        setState(true);
                      }}
                      class="flex w-full items-center justify-center rounded-full border border-green-500 bg-transparent px-2 text-lg text-green-500"
                    >
                      <RiSystemAddFill class="h-5 w-5 fill-current" />
                      <p> Create New Theme </p>
                    </MenuItem>
                  </div>
                )}
              </Menu>
            </PopoverPanel>
          </Show>
        </div>
      )}
    </Popover>
  );
};

export default BookmarkPopover;
