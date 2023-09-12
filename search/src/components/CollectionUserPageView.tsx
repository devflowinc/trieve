import { FiTrash } from "solid-icons/fi";
import {
  isCardCollectionPageDTO,
  type CardCollectionDTO,
  type UserDTO,
  type UserDTOWithVotesAndCards,
} from "../../utils/apiTypes";
import { For, Setter, Show, createEffect, createSignal } from "solid-js";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { getLocalTime } from "./CardMetadataDisplay";

export interface CollectionUserPageViewProps {
  user: UserDTOWithVotesAndCards | undefined;
  loggedUser: UserDTO | undefined;
  setOnDelete: Setter<() => void>;
  setShowConfirmModal: Setter<boolean>;
  initialCollections?: CardCollectionDTO[];
  initialCollectionPageCount?: number;
}

export const CollectionUserPageView = (props: CollectionUserPageViewProps) => {
  const api_host = import.meta.env.PUBLIC_API_HOST as string;
  const [collections, setCollections] = createSignal<CardCollectionDTO[]>([]);
  const [collectionPage, setCollectionPage] = createSignal(1);
  const [collectionPageCount, setCollectionPageCount] = createSignal(1);
  const [deleting, setDeleting] = createSignal(false);

  props.initialCollections && setCollections(props.initialCollections);
  props.initialCollectionPageCount &&
    setCollectionPageCount(props.initialCollectionPageCount);

  createEffect(() => {
    const userId = props.user?.id;
    if (userId === undefined) return;

    void fetch(`${api_host}/user/collections/${userId}/${collectionPage()}`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isCardCollectionPageDTO(data)) {
            setCollections(data.collections);
            setCollectionPageCount(
              data.total_pages == 0 ? 1 : data.total_pages,
            );
          } else {
            console.error("Invalid response", data);
          }
        });
      }
    });
  });

  const deleteCollection = (collection: CardCollectionDTO) => {
    if (props.user?.id !== collection.author_id) return;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${api_host}/card_collection`, {
          method: "DELETE",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            collection_id: collection.id,
          }),
        }).then((response) => {
          if (response.ok) {
            setDeleting(false);
            setCollections((prev) => {
              return prev.filter((c) => c.id != collection.id);
            });
          }
          if (response.status == 403) {
            setDeleting(false);
          }
          if (response.status == 401) {
            setDeleting(false);
          }
        });
      };
    });

    props.setShowConfirmModal(true);
  };
  return (
    <Show when={props.user !== undefined}>
      <div>
        <div class="mx-auto w-full text-center text-2xl font-bold">
          {props.user?.username ?? props.user?.email}'s Collections
        </div>
        <div class="mt-2 flow-root">
          <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
            <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
              <table class="min-w-full divide-y divide-gray-300 dark:divide-gray-700">
                <thead>
                  <tr>
                    <th
                      scope="col"
                      class="py-3.5 pl-4 pr-3 text-left text-base font-semibold dark:text-white sm:pl-[18px]"
                    >
                      Name
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-left text-base font-semibold dark:text-white"
                    >
                      Description
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-center text-base font-semibold dark:text-white"
                    >
                      Private
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-left text-base font-semibold dark:text-white"
                    >
                      Created at
                    </th>
                    <Show
                      when={
                        props.loggedUser != undefined &&
                        props.loggedUser.id == collections()[0]?.author_id
                      }
                    >
                      <th
                        scope="col"
                        class="relative hidden py-3.5 pl-3 pr-4 sm:pr-0"
                      >
                        <span class="sr-only">Delete</span>
                      </th>
                    </Show>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 dark:divide-gray-800">
                  <For each={collections()}>
                    {(collection) => (
                      <tr>
                        <td class="cursor-pointer whitespace-nowrap py-4 pl-4 pr-3 text-sm font-semibold text-gray-900 dark:text-white">
                          <a
                            class="w-full underline"
                            href={`/collection/${collection.id}`}
                          >
                            {collection.name}
                          </a>
                        </td>
                        <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-900 dark:text-gray-300">
                          {collection.description}
                        </td>
                        <td class="whitespace-nowrap px-3 py-4 text-center text-sm text-gray-900 dark:text-gray-300">
                          {!collection.is_public ? "✓" : ""}
                        </td>
                        <td class="whitespace-nowrap px-3 py-4 text-left text-sm text-gray-900 dark:text-gray-300">
                          {getLocalTime(
                            collection.created_at,
                          ).toLocaleDateString() +
                            " " +
                            //remove seconds from time
                            getLocalTime(collection.created_at)
                              .toLocaleTimeString()
                              .replace(/:\d+\s/, " ")}
                        </td>
                        <Show
                          when={
                            props.user != undefined &&
                            props.user.id == collection.author_id
                          }
                        >
                          <td
                            classList={{
                              "relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0":
                                true,
                              "hidden block":
                                props.loggedUser == undefined ||
                                props.loggedUser.id != collection.author_id,
                            }}
                          >
                            <button
                              classList={{
                                "h-fit text-red-700 dark:text-red-400": true,
                                "animate-pulse": deleting(),
                              }}
                              onClick={() => deleteCollection(collection)}
                            >
                              <FiTrash class="h-5 w-5" />
                            </button>
                          </td>
                        </Show>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </div>
        </div>
        <div class="mt-4 flex items-center justify-between">
          <div />
          <div class="flex items-center">
            <div class="text-sm text-neutral-400">
              {collectionPage()} / {collectionPageCount()}
            </div>
            <button
              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
              disabled={collectionPage() == 1}
              onClick={() => {
                setCollectionPage((prev) => prev - 1);
              }}
            >
              <BiRegularChevronLeft class="h-6 w-6 fill-current" />
            </button>
            <button
              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
              disabled={collectionPage() == collectionPageCount()}
              onClick={() => {
                setCollectionPage((prev) => prev + 1);
              }}
            >
              <BiRegularChevronRight class="h-6 w-6 fill-current" />
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
