import { useLocation, useNavigate, useParams } from "@solidjs/router";
import { Accessor, createContext, createMemo, useContext } from "solid-js";
import { JSX } from "solid-js";
import { DatasetAndUsage } from "trieve-ts-sdk";
import { UserContext } from "./UserContext";

type DatasetStore = {
  dataset: Accessor<DatasetAndUsage | null>;
  selectDataset: (id: string) => void;
  datasetId: string;
};

export const DatasetContext = createContext<DatasetStore>({
  dataset: () => null as unknown as DatasetAndUsage,
  selectDataset: (_id: string) => {},
  datasetId: "",
});

export const DatasetContextProvider = (props: { children: JSX.Element }) => {
  const params = useParams();
  const orgContext = useContext(UserContext);
  const navigate = useNavigate();

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

  const selectDataset = (id: string) => {
    // replace the pathname
    navigate(`/dataset/${id}`);
  };

  return (
    <DatasetContext.Provider
      value={{
        selectDataset,
        dataset: dataset,
        datasetId: params.id,
      }}
    >
      {props.children}
    </DatasetContext.Provider>
  );
};
