import { createSignal, useContext } from "solid-js";
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

const searchUiURL = import.meta.env.VITE_SEARCH_UI_URL as string;

export const DatasetHomepage = () => {
  const { datasetId } = useContext(DatasetContext);
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const [openSampleDataModal, setOpenSampleDataModal] =
    createSignal<boolean>(false);

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

  return (
    <div>
      <div class="flex items-end justify-between pb-2">
        <MagicSuspense skeletonHeight="36px" unstyled>
          <div class="flex items-center gap-2">
            <div class="text-lg font-medium">{datasetQuery.data?.name}</div>
            <CopyButton text={datasetId()} />
          </div>
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
          <div>Dataset ID: {datasetId()}</div>
          <div>Created At: {datasetQuery.data?.created_at}</div>
          <div>Chunk Count: {chunkCountQuery.data?.chunk_count}</div>
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
