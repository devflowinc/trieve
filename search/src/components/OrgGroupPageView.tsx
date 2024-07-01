import { FiTrash } from "solid-icons/fi";
import { isChunkGroupPageDTO, type ChunkGroupDTO } from "../../utils/apiTypes";
import {
  For,
  Setter,
  Show,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { getLocalTime } from "./ChunkMetadataDisplay";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { useDatasetServerConfig } from "../hooks/useDatasetServerConfig";
import { downloadFile } from "../utils/downloadFile";
import { FaSolidDownload } from "solid-icons/fa";

export interface GroupUserPageViewProps {
  setOnDelete: Setter<() => void>;
  setShowConfirmModal: Setter<boolean>;
}

export const GroupUserPageView = (props: GroupUserPageViewProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const $user = datasetAndUserContext.user;
  const [groups, setGroups] = createSignal<ChunkGroupDTO[]>([]);
  const [groupPage, setGroupPage] = createSignal(1);
  const [groupPageCount, setGroupPageCount] = createSignal(1);
  const [deleting, setDeleting] = createSignal(false);
  const serverConfig = useDatasetServerConfig();

  createEffect(() => {
    const userId = $user?.()?.id;
    if (userId === undefined) return;

    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    void fetch(
      `${apiHost}/dataset/groups/${currentDataset.dataset.id}/${groupPage()}`,
      {
        method: "GET",
        credentials: "include",
        headers: {
          "TR-Dataset": currentDataset.dataset.id,
        },
      },
    ).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isChunkGroupPageDTO(data)) {
            setGroups(data.groups);
            setGroupPageCount(data.total_pages == 0 ? 1 : data.total_pages);
          } else {
            console.error("Invalid response", data);
          }
        });
      }
    });
  });

  const deleteGroup = (group: ChunkGroupDTO) => {
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/chunk_group/${group.id}`, {
          method: "DELETE",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
            "TR-Dataset": currentDataset.dataset.id,
          },
        }).then((response) => {
          if (response.ok) {
            setDeleting(false);
            setGroups((prev) => {
              return prev.filter((c) => c.id != group.id);
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

  const handleDownloadFile = (group: ChunkGroupDTO) => {
    const datasetId = $dataset?.()?.dataset.id;
    if (group.file_id && datasetId) {
      void downloadFile(group.file_id, datasetId);
    }
  };

  return (
    <>
      <Show when={groups().length == 0}>
        <div class="text-center text-2xl font-bold">
          No groups found for this dataset
        </div>
      </Show>
      <div class="mx-auto w-full max-w-7xl">
        <div class="mx-auto w-full text-center text-2xl font-bold">
          {$dataset?.()?.dataset.name}'s Groups
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
                      class="px-3 py-3.5 text-left text-base font-semibold dark:text-white"
                    >
                      Created at
                    </th>
                    <Show when={$user?.() != undefined}>
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
                  <For each={groups()}>
                    {(group) => (
                      <tr>
                        <td class="cursor-pointer whitespace-nowrap text-wrap py-4 pl-4 pr-3 text-sm font-semibold text-gray-900 dark:text-white">
                          <a
                            class="w-full underline"
                            href={`/group/${group.id}`}
                          >
                            {group.name}
                          </a>
                        </td>
                        <td class="whitespace-nowrap text-wrap px-3 py-4 text-sm text-gray-900 dark:text-gray-300">
                          {group.description}
                        </td>
                        <td class="whitespace-nowrap px-3 py-4 text-left text-sm text-gray-900 dark:text-gray-300">
                          {getLocalTime(group.created_at).toLocaleDateString() +
                            " " +
                            //remove seconds from time
                            getLocalTime(group.created_at)
                              .toLocaleTimeString()
                              .replace(/:\d+\s/, " ")}
                        </td>
                        <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                          <div class="flex items-center gap-3">
                            <Show
                              when={
                                serverConfig()?.["DOCUMENT_DOWNLOAD_FEATURE"] &&
                                group.file_id
                              }
                            >
                              <button
                                title="Download uploaded file"
                                class="h-fit text-neutral-400 dark:text-neutral-300"
                                onClick={() => {
                                  handleDownloadFile(group);
                                }}
                              >
                                <FaSolidDownload />
                              </button>
                            </Show>
                            <button
                              classList={{
                                "h-fit text-red-700 dark:text-red-400": true,
                                "animate-pulse": deleting(),
                              }}
                              onClick={() => deleteGroup(group)}
                            >
                              <FiTrash class="h-5 w-5" />
                            </button>
                          </div>
                        </td>
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
              {groupPage()} / {groupPageCount()}
            </div>
            <button
              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
              disabled={groupPage() == 1}
              onClick={() => {
                setGroupPage((prev) => prev - 1);
              }}
            >
              <BiRegularChevronLeft class="h-6 w-6 fill-current" />
            </button>
            <button
              class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
              disabled={groupPage() == groupPageCount()}
              onClick={() => {
                setGroupPage((prev) => prev + 1);
              }}
            >
              <BiRegularChevronRight class="h-6 w-6 fill-current" />
            </button>
          </div>
        </div>
      </div>
    </>
  );
};
