import { useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { createQuery } from "@tanstack/solid-query";
import { useTrieve } from "../../hooks/useTrieve";
import { MagicBox, MagicSuspense } from "../../components/MagicBox";

export const DatasetHomepage = () => {
  const { datasetId } = useContext(DatasetContext);
  const trieve = useTrieve();

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

  return (
    <div class="p-4">
      <MagicSuspense skeletonHeight="36px" unstyled>
        <div class="pb-2 text-lg font-medium">{datasetQuery.data?.name}</div>
      </MagicSuspense>
      <MagicSuspense>
        <>
          <div>Dataset ID: {datasetId()}</div>
          <div>Created At: {datasetQuery.data?.created_at}</div>
          <div>Chunk Count: {chunkCountQuery.data?.chunk_count}</div>
        </>
      </MagicSuspense>
    </div>
  );
};
