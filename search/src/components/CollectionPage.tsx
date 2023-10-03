import { Show, createEffect, createSignal, For, onMount } from "solid-js";
import {
  isUserDTO,
  type CardCollectionDTO,
  type UserDTO,
  type CardCollectionBookmarkDTO,
  CardBookmarksDTO,
  BookmarkDTO,
  ScoreCardDTO,
  CardCollectionSearchDTO,
  isScoreCardDTO,
  isCardCollectionPageDTO,
  CardMetadata,
} from "../../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import {
  BiRegularLogInCircle,
  BiRegularQuestionMark,
  BiRegularX,
  BiRegularXCircle,
} from "solid-icons/bi";
import { FiEdit, FiLock, FiTrash } from "solid-icons/fi";
import { ConfirmModal } from "./Atoms/ConfirmModal";
import { PaginationController } from "./Atoms/PaginationController";
import { ScoreCardArray } from "./ScoreCardArray";
import SearchForm from "./SearchForm";
import type { Filters } from "./ResultsPage";
import CardMetadataDisplay from "./CardMetadataDisplay";
import { TbRobot } from "solid-icons/tb";

export interface CollectionPageProps {
  collectionID: string;
  defaultCollectionCards: {
    metadata: CardCollectionBookmarkDTO | CardCollectionSearchDTO;
    status: number;
  };
  page: number;
  query: string;
  searchType: string;
  dataTypeFilters: Filters;
}

export const CollectionPage = (props: CollectionPageProps) => {
  const apiHost: string = import.meta.env.PUBLIC_API_HOST as string;
  const cardMetadatasWithVotes: BookmarkDTO[] = [];
  const searchCardMetadatasWithVotes: ScoreCardDTO[] = [];
  const dataTypeFilters = encodeURIComponent(
    // eslint-disable-next-line solid/reactivity
    props.dataTypeFilters.dataTypes.join(","),
  );
  // eslint-disable-next-line solid/reactivity
  const linkFilters = encodeURIComponent(props.dataTypeFilters.links.join(","));

  // Sometimes this will error server-side if the collection is private so we have to handle it
  try {
    if (
      props.defaultCollectionCards.metadata.bookmarks.length > 0 &&
      !isScoreCardDTO(props.defaultCollectionCards.metadata.bookmarks[0])
    ) {
      cardMetadatasWithVotes.push(
        ...(props.defaultCollectionCards.metadata.bookmarks as BookmarkDTO[]),
      );
    } else if (
      props.defaultCollectionCards.metadata.bookmarks.length > 0 &&
      isScoreCardDTO(props.defaultCollectionCards.metadata.bookmarks[0])
    ) {
      searchCardMetadatasWithVotes.push(
        ...(props.defaultCollectionCards.metadata.bookmarks as ScoreCardDTO[]),
      );
    }
  } catch (e) {
    console.error(e);
  }

  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [metadatasWithVotes, setMetadatasWithVotes] = createSignal<
    BookmarkDTO[]
  >(cardMetadatasWithVotes);
  const [searchMetadatasWithVotes, setSearchMetadatasWithVotes] = createSignal<
    ScoreCardDTO[]
  >(searchCardMetadatasWithVotes);
  const [clientSideRequestFinished, setClientSideRequestFinished] =
    createSignal(false);
  const [collectionInfo, setCollectionInfo] = createSignal<CardCollectionDTO>(
    props.defaultCollectionCards.metadata.collection,
  );
  const [cardCollections, setCardCollections] = createSignal<
    CardCollectionDTO[]
  >([]);
  const [bookmarks, setBookmarks] = createSignal<CardBookmarksDTO[]>([]);
  const [error, setError] = createSignal("");
  const [fetchingCollections, setFetchingCollections] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [editing, setEditing] = createSignal(false);
  const [user, setUser] = createSignal<UserDTO | undefined>();
  const [totalPages, setTotalPages] = createSignal(
    props.defaultCollectionCards.metadata.total_pages,
  );
  const [loadingRecommendations, setLoadingRecommendations] =
    createSignal(false);
  const [recommendedCards, setRecommendedCards] = createSignal<CardMetadata[]>(
    [],
  );

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

  const [collectionQuery, setCollectionQuery] = createSignal("");
  const [streamingCollectionInference, setStreamingCollectionInference] =
    createSignal(false);
  const [collectionInference, setCollectionInference] = createSignal("");

  // Fetch the user info for the auth'ed user
  createEffect(() => {
    const abortController = new AbortController();

    void fetch(`${apiHost}/auth`, {
      method: "GET",
      credentials: "include",
      signal: abortController.signal,
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          isUserDTO(data) ? setUser(data) : setUser(undefined);
        });
      }

      if (response.status == 401) {
        setUser(undefined);
      }
    });

    return () => {
      abortController.abort();
    };
  });

  createEffect(() => {
    const abortController = new AbortController();
    let collection_id: string | null = null;
    if (props.query === "") {
      void fetch(
        `${apiHost}/card_collection/${props.collectionID}/${props.page}`,
        {
          method: "GET",
          credentials: "include",
          signal: abortController.signal,
        },
      ).then((response) => {
        if (response.ok) {
          void response.json().then((data) => {
            const collectionBookmarks = data as CardCollectionBookmarkDTO;
            collection_id = collectionBookmarks.collection.id;
            setCollectionInfo(collectionBookmarks.collection);
            setTotalPages(collectionBookmarks.total_pages);
            setMetadatasWithVotes(collectionBookmarks.bookmarks);
            setError("");
          });
        }
        if (response.status == 403) {
          setError("You are not authorized to view this collection");
        }
        if (response.status == 404) {
          setError("Collection not found, it never existed or was deleted");
        }
        if (response.status == 401) {
          setError(
            "You must be logged in and authorized to view this collection",
          );
          setShowNeedLoginModal(true);
        }
        setClientSideRequestFinished(true);
      });
    } else {
      void fetch(
        `${apiHost}/card_collection/${props.searchType}/${props.page}`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          signal: abortController.signal,
          credentials: "include",
          body: JSON.stringify({
            content: props.query,
            tag_set: props.dataTypeFilters.dataTypes,
            link: props.dataTypeFilters.links,
            collection_id: props.collectionID,
          }),
        },
      ).then((response) => {
        if (response.ok) {
          void response.json().then((data) => {
            const collectionBookmarks = data as CardCollectionSearchDTO;
            collection_id = collectionBookmarks.collection.id;
            setCollectionInfo(collectionBookmarks.collection);
            setTotalPages(collectionBookmarks.total_pages);
            setSearchMetadatasWithVotes(collectionBookmarks.bookmarks);
            setError("");
          });
        }
        if (response.status == 403) {
          setError("You are not authorized to view this collection");
        }
        if (response.status == 401) {
          setShowNeedLoginModal(true);
        }
        setClientSideRequestFinished(true);
      });

      return () => {
        abortController.abort();
      };
    }

    fetchCardCollections();

    setOnCollectionDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/card_collection`, {
          method: "DELETE",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
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

  onMount(() => {
    fetchBookmarks();
  });

  // Fetch the card collections for the auth'ed user
  const fetchCardCollections = () => {
    if (!user()) return;
    void fetch(`${apiHost}/card_collection/1`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isCardCollectionPageDTO(data)) {
            setCardCollections(data.collections);
            setTotalCollectionPages(data.total_pages);
          }
        });
      }
    });
  };

  const fetchBookmarks = () => {
    void fetch(`${apiHost}/card_collection/bookmark`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        card_ids: metadatasWithVotes().flatMap((m) => {
          return m.metadata.map((c) => c.id);
        }),
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setBookmarks(data as CardBookmarksDTO[]);
        });
      }
    });
  };

  const updateCollection = () => {
    setFetchingCollections(true);
    const body = {
      collection_id: collectionInfo().id,
      name: collectionInfo().name,
      description: collectionInfo().description,
      is_public: collectionInfo().is_public,
    };
    void fetch(`${apiHost}/card_collection`, {
      method: "PUT",
      credentials: "include",
      body: JSON.stringify(body),
      headers: {
        "Content-Type": "application/json",
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
    prev_recommendations: CardMetadata[],
  ) => {
    setLoadingRecommendations(true);
    void fetch(`${apiHost}/card/recommend`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        positive_card_ids: ids,
        limit: prev_recommendations.length + 10,
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const typed_data = data as CardMetadata[];
          const deduped_data = typed_data.filter((d) => {
            return !prev_recommendations.some((c) => c.id == d.id);
          });
          const new_recommendations = [
            ...prev_recommendations,
            ...deduped_data,
          ];
          setLoadingRecommendations(false);
          setRecommendedCards(new_recommendations);
        });
      }
    });
  };

  const fetchCollectionInference = async (
    collection_id: string,
    page: number,
  ) => {
    setCollectionInference("");

    try {
      const response = await fetch(`${apiHost}/card_collection/generate`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          collection_id: collection_id,
          page: page,
          query: collectionQuery(),
        }),
      });

      const reader = response.body?.getReader();
      setStreamingCollectionInference(true);

      if (!reader) return;

      let done = false;
      while (!done) {
        const { value, done: readerDone } = await reader.read();
        if (readerDone) {
          done = readerDone;
          setStreamingCollectionInference(false);
          continue;
        }

        const decoder = new TextDecoder();
        const chunk = decoder.decode(value);
        setCollectionInference((prev) => prev + chunk);
      }
    } catch (e) {
      console.error(e);
      setStreamingCollectionInference(false);
    }
  };

  const resizeTextarea = (textarea: HTMLTextAreaElement | null) => {
    if (!textarea) return;

    textarea.style.height = `${textarea.scrollHeight}px`;
    setCollectionQuery(textarea.value);
  };

  createEffect(() => {
    resizeTextarea(
      document.getElementById(
        "collection-query-textarea",
      ) as HTMLTextAreaElement | null,
    );
  });

  return (
    <>
      <Show
        when={
          !props.defaultCollectionCards.metadata.collection.is_public &&
          !clientSideRequestFinished()
        }
      >
        <div class="flex w-full flex-col items-center justify-center space-y-4">
          <div class="animate-pulse text-xl">Loading collection...</div>
          <div
            class="text-primary inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-current border-magenta border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]"
            role="status"
          >
            <span class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">
              Loading...
            </span>
          </div>
        </div>
      </Show>
      <Show
        when={
          props.defaultCollectionCards.metadata.collection.is_public ||
          clientSideRequestFinished()
        }
      >
        <div class="flex w-full flex-col items-center space-y-2">
          <Show when={error().length == 0}>
            <div class="flex w-full max-w-6xl items-center justify-end space-x-2 px-4 sm:px-8 md:px-20">
              <Show
                when={
                  !props.defaultCollectionCards.metadata.collection.is_public
                }
              >
                <FiLock class="text-green-500" />
              </Show>
              <Show
                when={cardCollections().some(
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
              <Show
                when={collectionInfo().description.length > 0 && !editing()}
              >
                <div class="mx-auto flex max-w-[300px] justify-items-center gap-x-2 md:max-w-fit">
                  <div class="text-center text-lg font-semibold">
                    Description:
                  </div>
                  <div class="line-clamp-1 flex w-full justify-start text-center text-lg">
                    {collectionInfo().description}
                  </div>
                </div>
              </Show>
              <div class="flex w-full max-w-6xl flex-col items-center justify-end space-x-2 px-4 pb-10 sm:px-8 md:px-20">
                <div class="mt-4 flex w-full max-w-[calc(100%-32px)] justify-center space-x-2 rounded-md bg-neutral-100 px-4 py-1 pr-[10px] dark:bg-neutral-700 min-[360px]:max-w-[calc(100%-64px)]">
                  <Show when={!props.query}>
                    <TbRobot class="mt-1 h-6 w-6" />
                  </Show>
                  <textarea
                    id="collection-query-textarea"
                    class="mr-2 h-fit max-h-[240px] w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
                    placeholder="Prompt the AI to generate text based on the cards in this collection..."
                    value={collectionQuery()}
                    onInput={(e) => {
                      resizeTextarea(e.target);
                    }}
                    onKeyDown={(e) => {
                      if (
                        ((e.ctrlKey || e.metaKey) && e.key === "Enter") ||
                        (!e.shiftKey && e.key === "Enter")
                      ) {
                        e.preventDefault();
                        e.stopPropagation();
                        void fetchCollectionInference(
                          props.collectionID,
                          props.page,
                        );
                      }
                    }}
                    rows="1"
                  />
                  <Show when={collectionQuery()}>
                    <button
                      classList={{
                        "pt-[2px]": !!props.query,
                      }}
                      onClick={(e) => {
                        e.preventDefault();
                        setCollectionQuery("");
                      }}
                    >
                      <BiRegularX class="h-7 w-7 fill-current" />
                    </button>
                  </Show>
                  <Show when={props.query}>
                    <button
                      classList={{
                        "border-l border-neutral-600 pl-[10px] dark:border-neutral-200":
                          !!collectionQuery(),
                      }}
                      type="submit"
                    >
                      <BiRegularQuestionMark class="mt-1 h-6 w-6 fill-current" />
                    </button>
                  </Show>
                </div>
                <Show
                  when={streamingCollectionInference() || collectionInference()}
                >
                  <div class="my-4 h-2 bg-neutral-500" />
                  <Show when={!collectionInference()}>
                    <img
                      src="/cooking-crab.gif"
                      class="aspect-square w-[128px]"
                      alt="cooking crab loading animation"
                    />
                  </Show>
                  <Show when={collectionInference()}>
                    <div
                      class="mx-auto w-full max-w-[calc(100%-32px)] min-[360px]:max-w-[calc(100%-64px)]"
                      innerText={collectionInference()}
                    />
                  </Show>
                </Show>
              </div>
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
                <span class="text-md font-semibold">Private?: </span>
                <input
                  type="checkbox"
                  checked={!collectionInfo().is_public}
                  onChange={(e) => {
                    setCollectionInfo({
                      ...collectionInfo(),
                      is_public: !e.target.checked,
                    });
                  }}
                  class="mt-1 h-4 w-4 items-center justify-start rounded-sm	border-gray-300 bg-neutral-500 align-middle accent-turquoise focus:ring-neutral-200 dark:border-neutral-700 dark:focus:ring-neutral-600"
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
                    filters={props.dataTypeFilters}
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
              {(card) => (
                <div class="mt-4">
                  <ScoreCardArray
                    totalCollectionPages={totalCollectionPages()}
                    signedInUserId={user()?.id}
                    cards={card.metadata}
                    score={isScoreCardDTO(card) ? card.score : 0}
                    collection={true}
                    setShowModal={setShowNeedLoginModal}
                    cardCollections={cardCollections()}
                    bookmarks={bookmarks()}
                    setOnDelete={setOnDelete}
                    setShowConfirmModal={setShowConfirmDeleteModal}
                    showExpand={clientSideRequestFinished()}
                    setCardCollections={setCardCollections}
                  />
                </div>
              )}
            </For>
            <div class="mx-auto my-12 flex items-center justify-center space-x-2">
              <PaginationController
                prefix={props.query ? "&" : "?"}
                query={
                  `/collection/${props.collectionID}` +
                  (props.query ? `?q=${props.query}` : "") +
                  (dataTypeFilters ? `&datatypes=${dataTypeFilters}` : "") +
                  (linkFilters ? `&links=${linkFilters}` : "") +
                  (props.searchType == "fulltextsearch"
                    ? `&searchType=fulltextsearch`
                    : "")
                }
                page={props.page}
                totalPages={totalPages()}
              />
            </div>
            <Show when={recommendedCards().length > 0}>
              <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] min-[360px]:max-w-[calc(100%-64px)]">
                <div class="flex w-full flex-col items-center rounded-md p-2">
                  <div class="text-xl font-semibold">Related Cards</div>
                </div>
                <For each={recommendedCards()}>
                  {(card) => (
                    <>
                      <div class="mt-4">
                        <CardMetadataDisplay
                          totalCollectionPages={totalCollectionPages()}
                          signedInUserId={user()?.id}
                          viewingUserId={user()?.id}
                          card={card}
                          cardCollections={cardCollections()}
                          bookmarks={bookmarks()}
                          setShowModal={setShowNeedLoginModal}
                          setShowConfirmModal={setShowConfirmDeleteModal}
                          fetchCardCollections={fetchCardCollections}
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
                      recommendedCards(),
                    )
                  }
                >
                  {recommendedCards().length == 0 ? "Get" : "Get More"} Related
                  Cards
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
                  This collection is currently empty
                </div>
              </div>
            </Show>
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
                Login or register to bookmark cards, vote, or view private
                collections
              </div>
              <div class="mx-auto flex w-fit flex-col space-y-3">
                <a
                  class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                  href="/auth/register"
                >
                  Register
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
          message="Are you sure you want to delete this card?"
        />
        <ConfirmModal
          showConfirmModal={showConfirmCollectionDeleteModal}
          setShowConfirmModal={setShowConfirmCollectionmDeleteModal}
          onConfirm={onCollectionDelete}
          message="Are you sure you want to delete this collection?"
        />
      </Show>
    </>
  );
};
