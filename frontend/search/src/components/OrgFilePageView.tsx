import { FiTrash } from "solid-icons/fi";
import { ChunkFile, FileAndGroupId } from "../utils/apiTypes";
import {
  For,
  Match,
  Setter,
  Show,
  Switch,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import { BiRegularChevronLeft, BiRegularChevronRight } from "solid-icons/bi";
import { getLocalTime } from "./ChunkMetadataDisplay";
import { A } from "@solidjs/router";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";

export interface FileUserPageViewProps {
  setOnDelete: Setter<() => void>;
  setShowConfirmModal: Setter<boolean>;
}

export const OrgFileViewPage = (props: FileUserPageViewProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);
  const user = datasetAndUserContext.user;

  const $dataset = datasetAndUserContext.currentDataset;
  const [loading, setLoading] = createSignal(true);
  const [fileAndGroupIds, setFileAndGroupIds] = createSignal<FileAndGroupId[]>(
    [],
  );
  const [filePage, setFilePage] = createSignal(1);
  const [filePageCount, setFilePageCount] = createSignal(1);
  const [deleting, setDeleting] = createSignal(false);

  createEffect(() => {
    const userId = user?.()?.id;
    if (userId === undefined) return;

    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    setLoading(true);
    void fetch(
      `${apiHost}/dataset/files/${currentDataset.dataset.id}/${filePage()}`,
      {
        method: "GET",
        credentials: "include",
        headers: {
          "TR-Dataset": currentDataset.dataset.id,
        },
      },
    ).then((response) => {
      if (response.ok) {
        void response
          .json()
          .then(
            (data: {
              file_and_group_ids: FileAndGroupId[];
              total_pages: number;
            }) => {
              setFileAndGroupIds(data.file_and_group_ids);
              setFilePageCount(data.total_pages == 0 ? 1 : data.total_pages);
            },
          );
      }

      setLoading(false);
    });
  });

  const deleteFile = (file: ChunkFile) => {
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/file/${file.id}?delete_chunks=true`, {
          method: "DELETE",
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
            "TR-Dataset": currentDataset.dataset.id,
          },
        }).then((response) => {
          if (response.ok) {
            setDeleting(false);
            setFileAndGroupIds((prev) => {
              return prev.filter((c) => c.file.id != file.id);
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
    <>
      <Switch>
        <Match when={loading()}>
          <div class="animate-pulse text-center text-2xl font-bold">
            Loading...
          </div>
        </Match>
        <Match when={fileAndGroupIds().length == 0}>
          <div class="text-center text-2xl font-bold">
            No files found for this dataset
          </div>
        </Match>
        <Match when={user?.() !== undefined && fileAndGroupIds().length > 0}>
          <div>
            <div class="mx-auto w-full text-center text-2xl font-bold">
              {$dataset?.()?.dataset.name}'s Files
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
                          Created at
                        </th>
                        <Show when={user?.() != undefined}>
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
                      <For each={fileAndGroupIds()}>
                        {(fileAndGroupId) => (
                          <tr>
                            <td class="cursor-pointer whitespace-nowrap py-4 pl-4 pr-3 text-sm font-semibold text-gray-900 dark:text-white">
                              <A
                                class="w-full underline"
                                href={`/group/${fileAndGroupId.group_id}`}
                              >
                                {fileAndGroupId.file.file_name}
                              </A>
                            </td>

                            <td class="whitespace-nowrap px-3 py-4 text-left text-sm text-gray-900 dark:text-gray-300">
                              {getLocalTime(
                                fileAndGroupId.file.created_at ?? "",
                              ).toLocaleDateString() +
                                " " +
                                //remove seconds from time
                                getLocalTime(fileAndGroupId.file.created_at)
                                  .toLocaleTimeString()
                                  .replace(/:\d+\s/, " ")}
                            </td>
                            <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                              <button
                                classList={{
                                  "h-fit text-red-700 dark:text-red-400": true,
                                  "animate-pulse": deleting(),
                                }}
                                onClick={() => deleteFile(fileAndGroupId.file)}
                              >
                                <FiTrash class="h-5 w-5" />
                              </button>
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
                  {filePage()} / {filePageCount()}
                </div>
                <button
                  class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                  disabled={filePage() == 1}
                  onClick={() => {
                    setFilePage((prev) => prev - 1);
                  }}
                >
                  <BiRegularChevronLeft class="h-6 w-6 fill-current" />
                </button>
                <button
                  class="disabled:text-neutral-400 dark:disabled:text-neutral-500"
                  disabled={filePage() == filePageCount()}
                  onClick={() => {
                    setFilePage((prev) => prev + 1);
                  }}
                >
                  <BiRegularChevronRight class="h-6 w-6 fill-current" />
                </button>
              </div>
            </div>
          </div>
        </Match>
      </Switch>
    </>
  );
};
