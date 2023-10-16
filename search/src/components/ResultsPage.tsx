import { Show, createEffect, createSignal, For, onMount } from "solid-js";
import {
  isUserDTO,
  type CardCollectionDTO,
  type CardsWithTotalPagesDTO,
  type ScoreCardDTO,
  type UserDTO,
  CardBookmarksDTO,
  isCardCollectionPageDTO,
} from "../../utils/apiTypes";
import { BiRegularLogIn, BiRegularXCircle } from "solid-icons/bi";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { PaginationController } from "./Atoms/PaginationController";
import { ConfirmModal } from "./Atoms/ConfirmModal";
import { ScoreCardArray } from "./ScoreCardArray";
import { Portal } from "solid-js/web";
import { AiOutlineRobot } from "solid-icons/ai";
import ChatPopup from "./ChatPopup";
import { IoDocumentOutline, IoDocumentsOutline } from "solid-icons/io";
export interface Filters {
  tagSet: string[];
  link: string[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  metadataFilters: any;
}
export interface ResultsPageProps {
  query: string;
  page: number;
  defaultResultCards: CardsWithTotalPagesDTO;
  filters: Filters;
  searchType: string;
}

const ResultsPage = (props: ResultsPageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const initialResultCards = props.defaultResultCards.score_cards;
  const initialTotalPages = props.defaultResultCards.total_card_pages;

  const [cardCollections, setCardCollections] = createSignal<
    CardCollectionDTO[]
  >([]);
  const [user, setUser] = createSignal<UserDTO | undefined>();
  const [resultCards, setResultCards] =
    createSignal<ScoreCardDTO[]>(initialResultCards);
  const [clientSideRequestFinished, setClientSideRequestFinished] =
    createSignal(false);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});
  const [bookmarks, setBookmarks] = createSignal<CardBookmarksDTO[]>([]);
  const [totalPages, setTotalPages] = createSignal(initialTotalPages);
  const [openChat, setOpenChat] = createSignal(false);
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);

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
        card_ids: resultCards().flatMap((c) => {
          return c.metadata.map((m) => m.id);
        }),
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const cardBookmarks = data as CardBookmarksDTO[];
          setBookmarks(cardBookmarks);
        });
      }
    });
  };

  // Fetch the user info for the auth'ed user
  createEffect(() => {
    void fetch(`${apiHost}/auth`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          isUserDTO(data) ? setUser(data) : setUser(undefined);
        });
        return;
      }
    });
  });

  createEffect(() => {
    const abortController = new AbortController();

    void fetch(`${apiHost}/card/${props.searchType}/${props.page}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      signal: abortController.signal,
      body: JSON.stringify({
        content: props.query,
        tag_set: props.filters.tagSet,
        link: props.filters.link,
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        filters: props.filters.metadataFilters,
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          const result = data.score_cards as ScoreCardDTO[];
          setResultCards(result);
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setTotalPages(data.total_card_pages);
          setClientSideRequestFinished(true);
        });
      } else {
        setClientSideRequestFinished(true);
      }
    });

    fetchCardCollections();

    return () => {
      abortController.abort();
    };
  });

  onMount(() => {
    fetchBookmarks();
  });

  createEffect(() => {
    if (!openChat()) {
      setSelectedIds([]);
    }
  });

  return (
    <>
      <Show when={openChat()}>
        <Portal>
          <FullScreenModal isOpen={openChat} setIsOpen={setOpenChat}>
            <div class="max-h-[75vh] min-h-[75vh] min-w-[75vw] max-w-[75vw] overflow-y-auto scrollbar-thin">
              <ChatPopup
                selectedIds={selectedIds}
                setShowNeedLoginModal={setShowNeedLoginModal}
                setOpenChat={setOpenChat}
              />
            </div>
          </FullScreenModal>
        </Portal>
      </Show>
      <div class="mt-12 flex w-full flex-col items-center space-y-4">
        <Show when={resultCards().length === 0 && !clientSideRequestFinished()}>
          <div
            class="text-primary inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-current border-magenta border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]"
            role="status"
          >
            <span class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">
              Loading...
            </span>
          </div>
        </Show>
        <Show when={resultCards().length === 0 && clientSideRequestFinished()}>
          <button
            onClick={() => {
              window.location.href = `/search?q=${props.query}&page=${
                props.page + 1
              }`;
            }}
          >
            <div class="text-2xl">No results found</div>
          </button>
        </Show>
        <div class="flex w-full max-w-6xl flex-col space-y-4 px-1 min-[360px]:px-4 sm:px-8 md:px-20">
          <For each={resultCards()}>
            {(card) => (
              <div>
                <ScoreCardArray
                  totalCollectionPages={totalCollectionPages()}
                  signedInUserId={user()?.id}
                  cardCollections={cardCollections()}
                  cards={card.metadata}
                  score={card.score}
                  setShowModal={setShowNeedLoginModal}
                  bookmarks={bookmarks()}
                  setOnDelete={setOnDelete}
                  setShowConfirmModal={setShowConfirmDeleteModal}
                  showExpand={clientSideRequestFinished()}
                  setCardCollections={setCardCollections}
                  setSelectedIds={setSelectedIds}
                  selectedIds={selectedIds}
                />
              </div>
            )}
          </For>
        </div>
      </div>
      <div class="mx-auto my-12 flex items-center space-x-2">
        <PaginationController page={props.page} totalPages={totalPages()} />
      </div>
      <div>
        <div
          data-dial-init
          class="group fixed bottom-6 right-6"
          onMouseEnter={() => {
            document
              .getElementById("speed-dial-menu-text-outside-button")
              ?.classList.remove("hidden");
          }}
          onMouseLeave={() => {
            document
              .getElementById("speed-dial-menu-text-outside-button")
              ?.classList.add("hidden");
          }}
        >
          <div
            id="speed-dial-menu-text-outside-button"
            class="mb-4 flex hidden flex-col items-center space-y-2"
          >
            <button
              type="button"
              class="relative h-[52px] w-[52px] items-center justify-center rounded-lg border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:ring-4 focus:ring-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-white dark:focus:ring-gray-400"
              onClick={() => {
                setSelectedIds(
                  resultCards()
                    .flatMap((c) => {
                      return c.metadata.map((m) => m.id);
                    })
                    .slice(0, 10),
                );
                setOpenChat(true);
              }}
            >
              <IoDocumentsOutline class="mx-auto h-7 w-7" />
              <span class="font-sm absolute -left-[10.5rem] top-1/2 mb-px block -translate-y-1/2 break-words text-sm">
                Chat with all documents
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
              <span class="font-sm absolute -left-[12.85rem] top-1/2 mb-px block -translate-y-1/2 text-sm">
                Chat with selected documents
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
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold">
              Cannot use this feature without an account
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href="/auth/register"
              >
                Register
                <BiRegularLogIn class="h-6 w-6 fill-current" />
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
    </>
  );
};

export default ResultsPage;
