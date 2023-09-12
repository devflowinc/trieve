import { Show, createEffect, createMemo, createSignal } from "solid-js";
import {
  isUserDTO,
  type CardCollectionDTO,
  type CardMetadataWithVotes,
  type UserDTO,
  isCardMetadataWithVotes,
  SingleCardDTO,
  CardBookmarksDTO,
  isCardCollectionPageDTO,
} from "../../utils/apiTypes";
import ScoreCard from "./ScoreCard";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { BiRegularLogIn, BiRegularXCircle } from "solid-icons/bi";
import { ConfirmModal } from "./Atoms/ConfirmModal";

export interface SingleCardPageProps {
  cardId: string | undefined;
  defaultResultCard: SingleCardDTO;
}
export const SingleCardPage = (props: SingleCardPageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const initialCardMetadata = props.defaultResultCard.metadata;

  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [cardMetadata, setCardMetadata] =
    createSignal<CardMetadataWithVotes | null>(initialCardMetadata);
  const [error, setError] = createSignal("");
  const [fetching, setFetching] = createSignal(true);
  const [cardCollections, setCardCollections] = createSignal<
    CardCollectionDTO[]
  >([]);
  const [user, setUser] = createSignal<UserDTO | undefined>();
  const [bookmarks, setBookmarks] = createSignal<CardBookmarksDTO[]>([]);
  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});

  if (props.defaultResultCard.status == 401) {
    setError("You are not authorized to view this card.");
  }
  if (props.defaultResultCard.status == 404) {
    setError("This card could not be found.");
  }

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

  createEffect(() => {
    fetchCardCollections();
    fetchBookmarks();
  });

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

  const fetchBookmarks = () => {
    void fetch(`${apiHost}/card_collection/bookmark`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        card_ids: cardMetadata()?.id ? [cardMetadata()?.id] : [],
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setBookmarks(data as CardBookmarksDTO[]);
        });
      }
    });
  };

  createEffect(() => {
    setFetching(true);
    void fetch(`${apiHost}/card/${props.cardId ?? ""}`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (!isCardMetadataWithVotes(data)) {
            setError("This card could not be found.");
            setFetching(false);
            return;
          }

          setCardMetadata(data);
          setError("");
          setFetching(false);
        });
      }
      if (response.status == 403) {
        setError("You are not authorized to view this card.");
        setFetching(false);
      }
      if (response.status == 404) {
        setError("This card could not be found.");
        setFetching(false);
      }
      if (response.status == 401) {
        setError("Sign in to view this card.");
        setShowNeedLoginModal(true);
      }
    });
  });

  const getCard = createMemo(() => {
    if (error().length > 0) {
      return null;
    }
    const curCardMetadata = cardMetadata();
    if (!curCardMetadata) {
      return null;
    }
    return (
      <ScoreCard
        totalCollectionPages={totalCollectionPages()}
        signedInUserId={user()?.id}
        card={curCardMetadata}
        score={0}
        setShowModal={setShowNeedLoginModal}
        cardCollections={cardCollections()}
        bookmarks={bookmarks()}
        setOnDelete={setOnDelete}
        setShowConfirmModal={setShowConfirmDeleteModal}
        initialExpanded={true}
      />
    );
  });

  return (
    <>
      <div class="mt-2 flex w-full flex-col items-center justify-center">
        <div class="flex w-full max-w-6xl flex-col justify-center px-4 sm:px-8 md:px-20">
          {getCard()}
          <Show when={error().length > 0 && !fetching()}>
            <div class="flex w-full flex-col items-center rounded-md p-2">
              <div class="text-xl font-bold text-red-500">{error()}</div>
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
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold">
              You must be signed in to vote, bookmark, or view this card it if
              it's private
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
