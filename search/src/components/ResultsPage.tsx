import { Show, createEffect, createSignal, For } from "solid-js";
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

export interface Filters {
  dataTypes: string[];
  links: string[];
}
export interface ResultsPageProps {
  query: string;
  page: number;
  defaultResultCards: CardsWithTotalPagesDTO;
  filters: Filters;
  searchType: string;
}

const ResultsPage = (props: ResultsPageProps) => {
  // eslint-disable-next-line solid/reactivity
  const dataTypeFilters = encodeURIComponent(props.filters.dataTypes.join(","));
  // eslint-disable-next-line solid/reactivity
  const linkFilters = encodeURIComponent(props.filters.links.join(","));
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const initialResultCards = props.defaultResultCards.score_cards;
  const totalPages = props.defaultResultCards.total_card_pages;
  const [cardCollections, setCardCollections] = createSignal<
    CardCollectionDTO[]
  >([]);
  const [user, setUser] = createSignal<UserDTO | undefined>();

  const [resultCards, setResultCards] =
    createSignal<ScoreCardDTO[]>(initialResultCards);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);

  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);

  const [collectionPage, setCollectionPage] = createSignal(1);
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});
  const [bookmarks, setBookmarks] = createSignal<CardBookmarksDTO[]>([]);

  const fetchCardCollections = () => {
    if (!user()) return;

    void fetch(`${apiHost}/card_collection/${collectionPage()}`, {
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
        filter_oc_file_path: props.filters.dataTypes,
        filter_link_url: props.filters.links,
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          const result = data.score_cards as ScoreCardDTO[];
          setResultCards(result);
        });
      }
    });

    fetchCardCollections();

    return () => {
      abortController.abort();
    };
  });

  createEffect(() => {
    fetchBookmarks();
  });

  return (
    <>
      <div class="mt-12 flex w-full flex-col items-center space-y-4">
        <Show when={resultCards().length === 0}>
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
                  collectionPage={collectionPage()}
                  setCollectionPage={setCollectionPage}
                  signedInUserId={user()?.id}
                  cardCollections={cardCollections()}
                  cards={card.metadata}
                  score={card.score}
                  setShowModal={setShowNeedLoginModal}
                  fetchCardCollections={fetchCardCollections}
                  fetchBookmarks={fetchBookmarks}
                  bookmarks={bookmarks()}
                  setOnDelete={setOnDelete}
                  setShowConfirmModal={setShowConfirmDeleteModal}
                />
              </div>
            )}
          </For>
        </div>
      </div>
      <div class="mx-auto my-12 flex items-center space-x-2">
        <PaginationController
          query={
            `/search?q=${props.query}` +
            (dataTypeFilters ? `&datatypes=${dataTypeFilters}` : "") +
            (linkFilters ? `&links=${linkFilters}` : "") +
            (props.searchType == "fulltextsearch"
              ? `&searchType=fulltextsearch`
              : "")
          }
          prefix="&"
          page={props.page}
          totalPages={totalPages}
        />
      </div>
      <Show when={showNeedLoginModal()}>
        <FullScreenModal
          isOpen={showNeedLoginModal}
          setIsOpen={setShowNeedLoginModal}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold">
              Cannot vote or use bookmarks without an account
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
