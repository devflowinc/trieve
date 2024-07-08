import { createMemo, useContext } from "solid-js";
import { DatasetContext } from "../contexts/DatasetContext";

export const DatasetName = () => {
  const datasetContext = useContext(DatasetContext);

  const curDatasetName = createMemo(() => {
    const dataset = datasetContext.dataset?.();
    if (!dataset) return null;
    return dataset.name;
  });

  return (
    <h3 class="text-xl font-semibold text-neutral-600">
      {curDatasetName()} Dataset
    </h3>
  );
};
