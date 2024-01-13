import { Show, createEffect, createSignal, For } from "solid-js";
import {
  type ChunkCollectionDTO,
  type UserDTOWithVotesAndChunks,
  ChunkBookmarksDTO,
  isChunkCollectionPageDTO,
} from "../../utils/apiTypes";
import ChunkMetadataDisplay, { getLocalTime } from "./ChunkMetadataDisplay";
import { PaginationController } from "./Atoms/PaginationController";
import { CollectionUserPageView } from "./CollectionUserPageView";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { BiRegularLogIn, BiRegularXCircle } from "solid-icons/bi";
import { ConfirmModal } from "./Atoms/ConfirmModal";
import { useStore } from "@nanostores/solid";
import { currentUser } from "../stores/userStore";
import { currentDataset } from "../stores/datasetStore";

export interface UserChunkDisplayProps {
  id: string;
  page: number;
  initialUser?: UserDTOWithVotesAndChunks | null;
  initialUserCollections?: ChunkCollectionDTO[];
  initialUserCollectionPageCount?: number;
}

export const UserChunkDisplay = (props: UserChunkDisplayProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const $dataset = useStore(currentDataset);

  const [user, setUser] = createSignal<UserDTOWithVotesAndChunks>();
  const [clientSideRequestFinished, setClientSideRequestFinished] =
    createSignal(false);
  const $currentUser = useStore(currentUser);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [showConfirmModal, setShowConfirmModal] = createSignal(false);
  const [chunkCollections, setChunkCollections] = createSignal<
    ChunkCollectionDTO[]
  >([]);

  props.initialUser && setUser(props.initialUser);

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal<() => void>(() => {});
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onCollectionDelete, setOnCollectionDelete] = createSignal(() => {});
  const [
    showConfirmCollectionDeleteModal,
    setShowConfirmCollectionmDeleteModal,
  ] = createSignal(false);
  const [bookmarks, setBookmarks] = createSignal<ChunkBookmarksDTO[]>([]);
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);

  createEffect(() => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    void fetch(`${apiHost}/user/${props.id}/${props.page}`, {
      method: "GET",
      headers: {
        "AF-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setUser(data as UserDTOWithVotesAndChunks);
        });
      }
      setClientSideRequestFinished(true);
    });
  });

  createEffect(() => {
    fetchChunkCollections();
    fetchBookmarks();
  });

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
        chunk_ids: user()?.chunks.map((c) => c.id)
          ? user()?.chunks.map((c) => c.id)
          : [],
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          setBookmarks(data as ChunkBookmarksDTO[]);
        });
      }
    });
  };

  const fetchChunkCollections = () => {
    if (!user()) return;
    const currentDataset = $dataset();
    if (!currentDataset) return;

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

  return (
    <>
      <Show when={user() == null && !clientSideRequestFinished()}>
        <div class="flex w-full flex-col items-center justify-center space-y-4">
          <div class="animate-pulse text-xl">Loading user...</div>
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
      <Show when={user() == null && clientSideRequestFinished()}>
        <div class="flex w-full flex-col items-center justify-center space-y-4">
          <div class="text-xl">User not found</div>
        </div>
      </Show>
      <Show when={user() != null}>
        <h1 class="line-clamp-1 break-all py-8 text-center text-lg font-bold min-[320px]:text-xl sm:text-4xl">
          {user()?.username ?? user()?.email ?? "User not found"}
        </h1>
        <div class="mx-auto grid w-fit grid-cols-[3fr,4fr] items-center justify-items-end gap-x-2 gap-y-2 text-end align-middle sm:grid-cols-[4fr,5fr] sm:gap-x-4">
          {user()?.website && (
            <>
              <div class="font-semibold">Website:</div>
              <a
                href={user()?.website ?? ""}
                target="_blank"
                class="line-clamp-1 flex w-full justify-start text-magenta-500 underline dark:text-turquoise-400"
              >
                {user()?.website}
              </a>
            </>
          )}
          {user()?.email && user()?.visible_email && (
            <>
              <div class="font-semibold">Email:</div>
              <div class="flex w-full justify-start break-all">
                {user()?.email}
              </div>
            </>
          )}
          <div class="font-semibold">Chunks Created:</div>
          <div class="flex w-full justify-start">
            <Show when={user() != null}>
              {user()?.total_chunks_created.toLocaleString()}
            </Show>
          </div>
          <div class="font-semibold">Date Joined:</div>
          <div class="flex w-full justify-start">
            {getLocalTime(user()?.created_at ?? "").toLocaleDateString()}
          </div>
        </div>
        <div class="mb-4 mt-4 flex  flex-col overflow-hidden border-t border-neutral-500 pt-4 text-xl">
          <CollectionUserPageView
            user={user()}
            initialCollections={props.initialUserCollections}
            initialCollectionPageCount={props.initialUserCollectionPageCount}
            loggedUser={$currentUser()}
            setOnDelete={setOnCollectionDelete}
            setShowConfirmModal={setShowConfirmCollectionmDeleteModal}
          />
        </div>
        <Show when={(user()?.chunks.length ?? 0) > 0}>
          <div class="mb-4 mt-4 flex flex-col border-t border-neutral-500 pt-4 text-xl">
            <span>Chunks:</span>
          </div>
          <div class="flex w-full flex-col space-y-4">
            <div class="flex w-full flex-col space-y-4">
              <For each={user()?.chunks}>
                {(chunk) => (
                  <div class="w-full">
                    <ChunkMetadataDisplay
                      totalCollectionPages={totalCollectionPages()}
                      setShowConfirmModal={setShowConfirmModal}
                      viewingUserId={props.id}
                      chunk={chunk}
                      setShowModal={setShowNeedLoginModal}
                      chunkCollections={chunkCollections()}
                      fetchChunkCollections={fetchChunkCollections}
                      setChunkCollections={setChunkCollections}
                      setOnDelete={setOnDelete}
                      bookmarks={bookmarks()}
                      showExpand={clientSideRequestFinished()}
                    />
                  </div>
                )}
              </For>
            </div>
          </div>
          <div class="mx-auto my-12 flex items-center justify-center space-x-2">
            <PaginationController
              page={props.page}
              totalPages={Math.ceil((user()?.total_chunks_created ?? 0) / 25)}
            />
          </div>
        </Show>
      </Show>
      <Show when={showNeedLoginModal()}>
        <FullScreenModal
          isOpen={showNeedLoginModal}
          setIsOpen={setShowNeedLoginModal}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold dark:text-white">
              You must be signed in to vote, bookmark, or view this chunk it if
              it's private
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href={`${apiHost}/auth?dataset_id=${
                  $dataset()?.dataset.name ?? ""
                }`}
              >
                Login/Register
                <BiRegularLogIn class="h-6 w-6 fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
      <ConfirmModal
        showConfirmModal={showConfirmModal}
        setShowConfirmModal={setShowConfirmModal}
        onConfirm={onDelete}
        message={"Are you sure you want to delete this chunk?"}
      />
      <ConfirmModal
        showConfirmModal={showConfirmCollectionDeleteModal}
        setShowConfirmModal={setShowConfirmCollectionmDeleteModal}
        onConfirm={onCollectionDelete}
        message={"Are you sure you want to delete this theme?"}
      />
    </>
  );
};
