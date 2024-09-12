import { useParams } from "@solidjs/router";
import { Accessor, createContext, createMemo, useContext } from "solid-js";
import { JSX } from "solid-js";
import { DatasetAndUsage } from "trieve-ts-sdk";
import { UserContext } from "./UserContext";

type DatasetStore = {
  dataset: Accessor<DatasetAndUsage | null>;
  datasetId: string;
};

export const DatasetContext = createContext<DatasetStore>({
  dataset: () => null as unknown as DatasetAndUsage,
  datasetId: "",
});

export const DatasetContextProvider = (props: { children: JSX.Element }) => {
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
        dataset: dataset,
        datasetId: datasetId,
      }}
    >
      {props.children}
    </DatasetContext.Provider>
  );
};
