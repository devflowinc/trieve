import {
  createEffect,
  createSignal,
  onCleanup,
  useContext,
  Show,
} from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { createQuery } from "@tanstack/solid-query";
import { useTrieve } from "../../hooks/useTrieve";
import { MagicSuspense } from "../../components/MagicBox";
import { AddSampleDataModal } from "../../components/DatasetExampleModal";
import { CopyButton } from "../../components/CopyButton";
import { UserContext } from "../../contexts/UserContext";
import { CodeExamples } from "../../components/CodeExamples";
import { Spacer } from "../../components/Spacer";
import { BuildingSomething } from "../../components/BuildingSomething";
import { TbReload } from "solid-icons/tb";
import { createToast } from "../../components/ShowToasts";
import { BiRegularPencil, BiRegularCheck, BiRegularX } from "solid-icons/bi";

const searchUiURL = import.meta.env.VITE_SEARCH_UI_URL as string;
const apiHost: string = import.meta.env.VITE_API_HOST as string;

export const DatasetHomepage = () => {
  const { datasetId } = useContext(DatasetContext);
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const [openSampleDataModal, setOpenSampleDataModal] =
    createSignal<boolean>(false);

  const [showEditTitle, setShowEditTitle] = createSignal<boolean>(false);
  const [newDatasetName, setNewDatasetName] = createSignal<string>("");

  const datasetQuery = createQuery(() => ({
    queryKey: ["dataset", datasetId()],
    queryFn: async () => {
      return trieve.fetch("/api/dataset/{dataset_id}", "get", {
        datasetId: datasetId(),
      });
    },
  }));

  const chunkCountQuery = createQuery(() => ({
    queryKey: ["dataset-chunk-count", datasetId()],
    queryFn: async () => {
      return trieve.fetch("/api/dataset/usage/{dataset_id}", "get", {
        datasetId: datasetId(),
      });
    },
  }));

  const orgDatasetParams = (datasetId: string) => {
    return datasetId
      ? `/?organization=${userContext.selectedOrg().id}&dataset=${datasetId}`
      : "";
  };

  const refetchChunkCount = async (showForDeltaZero: boolean) => {
    try {
      const currentUsage = chunkCountQuery.data;
      const prevCount = currentUsage?.chunk_count || 0;

      const newData = await chunkCountQuery.refetch();

      const newCount: number = newData.data?.chunk_count as number;
      const countDifference = newCount - prevCount;

      if (countDifference == 0 && !showForDeltaZero) {
        return;
      }

      createToast({
        title: "Updated",
        type: "success",
        message: `Successfully updated chunk count: ${countDifference} chunk${
          Math.abs(countDifference) === 1 ? " has" : "s have"
        } been ${
          countDifference > 0
            ? "added"
            : countDifference < 0
              ? "removed"
              : "added or removed"
        } since last update.`,
        timeout: 3000,
      });
    } catch (error) {
      createToast({
        title: "Error",
        type: "error",
        message: `Failed to reload chunk count: ${(error as Error).message}`,
      });
    }
  };

  createEffect(() => {
    const refreshChunkCountId = setInterval(
      () => void refetchChunkCount(false),
      30000,
    );

    onCleanup(() => clearInterval(refreshChunkCountId));
  });

  const editTitle = async () => {
    await fetch(`${apiHost}/dataset`, {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": datasetId(),
      },
      body: JSON.stringify({
        dataset_id: datasetId(),
        dataset_name: newDatasetName(),
      }),
    })
      .then(() => {
        createToast({
          title: "Success",
          type: "success",
          message: "Dataset name has been updated",
        });

        setShowEditTitle(false);
        datasetQuery.refetch().catch(() => {
          createToast({
            title: "Error",
            type: "error",
            message: "Could not fetch dataset name",
          });
        });
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message: "Failed to update dataset name",
        });
      });
  };

  const handleSaveTitle = () => {
    editTitle().catch((err) => {
      console.error(err);
    });
  };

  return (
    <div>
      <div class="flex items-end justify-between pb-2">
        <MagicSuspense skeletonHeight="36px" unstyled>
          <Show when={!showEditTitle()}>
            <div class="flex items-center justify-center gap-2 text-xl font-medium">
              {datasetQuery.data?.name}
              <button
                class="text-base opacity-80 hover:text-fuchsia-500"
                onClick={() => setShowEditTitle(true)}
              >
                <BiRegularPencil />
              </button>
            </div>
          </Show>
          <Show when={showEditTitle()}>
            <div class="align-center mb-1 flex flex-row gap-1">
              <input
                type="text"
                name="dataset-name"
                id="dataset-name"
                placeholder="Enter the new dataset name"
                class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                value={newDatasetName()}
                onInput={(e) => setNewDatasetName(e.currentTarget.value)}
              />
              <button class="text-base opacity-80 hover:text-green-500">
                <BiRegularCheck class="text-2xl" onClick={handleSaveTitle} />
              </button>
              <button
                class="text-base opacity-80 hover:text-red-500"
                onClick={() => setShowEditTitle(false)}
              >
                <BiRegularX class="text-2xl" />
              </button>
            </div>
          </Show>
        </MagicSuspense>
        <div class="flex gap-2">
          <a
            class="flex cursor-pointer items-center space-x-2 rounded-md border bg-magenta-500 px-2 py-1 text-sm text-white"
            href={`${searchUiURL}/upload${orgDatasetParams(datasetId())}`}
            target="_blank"
          >
            <p>Upload file(s)</p>
          </a>
          <button
            class="flex items-center space-x-2 rounded-md border bg-magenta-500 px-2 py-1 text-sm text-white"
            onClick={() => setOpenSampleDataModal(true)}
          >
            Add Sample Data
          </button>
        </div>
      </div>
      <MagicSuspense>
        <>
          <div class="flex content-center items-center gap-1.5">
            <span class="font-medium">Dataset ID:</span> {datasetId()}
            <CopyButton size={15} text={datasetId()} />
          </div>
          <div class="flex content-center items-center gap-1.5">
            <span class="font-medium">Organization ID:</span>{" "}
            {userContext.selectedOrg().id}{" "}
            <CopyButton size={15} text={userContext.selectedOrg().id} />
          </div>
          <div class="flex flex-row content-center items-center gap-1">
            <span class="font-medium">Chunk Count:</span>{" "}
            {chunkCountQuery.data?.chunk_count}
            <button
              class="text-sm opacity-80 hover:text-fuchsia-500"
              onClick={() => {
                void refetchChunkCount(true);
              }}
            >
              <TbReload />
            </button>
          </div>
        </>
      </MagicSuspense>
      <Spacer h={12} />
      <CodeExamples />
      <AddSampleDataModal
        addedDataCallback={() => {
          // mutateUsage((prev) => {
          //   if (prev)
          //     return {
          //       ...prev,
          //       chunk_count: SAMPLE_DATASET_SIZE,
          //     };
          // });
        }}
        openModal={openSampleDataModal}
        closeModal={() => setOpenSampleDataModal(false)}
      />
      <Spacer h={12} />
      <BuildingSomething />
    </div>
  );
};
