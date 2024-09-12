import { useParams } from "@solidjs/router";
import { Accessor, createContext, JSX, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { createMemo } from "solid-js/types/server/reactive.js";
import { DatasetAndUsage } from "trieve-ts-sdk";

interface Props {
  children?: JSX.Element;
}

type DatasetStore = {
  dataset: Accessor<DatasetAndUsage | null>;
  datasetId: string;
};

const DatasetContext = createContext<DatasetStore>({
  dataset: () => null as unknown as DatasetAndUsage,
  datasetId: "",
});

// Needs to ensure dataset and org don't desync
export const DatasetLayout = (props: Props) => {
  const datasetId = useParams().id;
  const orgContext = useContext(UserContext);

  const dataset = createMemo(() => {
    const possDatasets = orgContext.orgDatasets();
    if (possDatasets) {
      return (
        possDatasets.find((dataset) => dataset.dataset.id === datasetId) || null
      );
    } else {
      return null;
    }
  });

  return (
    <DatasetContext.Provider
      value={{
        dataset,
        datasetId,
      }}
    >
      <div class="grid max-h-full grow grid-cols-[200px_1fr] overflow-hidden bg-green-500">
        <div class="h-full bg-red-500">sidebar</div>
        <div class="overflow-scroll">{props.children}</div>
      </div>
    </DatasetContext.Provider>
  );
};
