import { useLocation, useNavigate, useParams } from "@solidjs/router";
import {
  Accessor,
  createContext,
  createEffect,
  createMemo,
  useContext,
} from "solid-js";
import { JSX } from "solid-js";
import { DatasetAndUsage } from "trieve-ts-sdk";
import { UserContext } from "./UserContext";

type DatasetStore = {
  dataset: Accessor<DatasetAndUsage | null>;
  selectDataset: (id: string) => void;
  datasetId: Accessor<string>;
};

export const DatasetContext = createContext<DatasetStore>({
  dataset: () => null as unknown as DatasetAndUsage,
  selectDataset: (_id: string) => {},
  datasetId: () => "" as unknown as string,
});

export const DatasetContextProvider = (props: { children: JSX.Element }) => {
  const params = useParams();
  const orgContext = useContext(UserContext);
  const location = useLocation();
  const navigate = useNavigate();

  createEffect(() => {
    const selectedOrg = orgContext.selectedOrg();
    const orgDatasets = orgContext.orgDatasets();
    if (!orgDatasets) {
      return;
    }
    if (selectedOrg) {
      console.log("selected org", selectedOrg);
      if (
        orgDatasets &&
        location.pathname.includes("/dataset") &&
        !orgDatasets.find((orgDataset) => orgDataset.dataset.id === params.id)
      ) {
        const firstDataset = orgDatasets[0]?.dataset;
        if (!firstDataset) {
          console.log("no first dataset");
          return navigate("/org");
        }
        selectDataset(firstDataset.id);
        // Set the new params in the url
      }
    }
  });

  const dataset = createMemo(() => {
    const possDatasets = orgContext.orgDatasets();
    if (Array.isArray(possDatasets)) {
      return (
        possDatasets.find((dataset) => dataset.dataset.id === params.id) ?? null
      );
    } else {
      return null;
    }
  });

  const selectDataset = (id: string) => {
    const curPath = location.pathname;
    // Get the part after the /dataset/
    if (curPath.includes(id)) {
      navigate(curPath);
      return;
    }

    navigate(`/dataset/${id}`);
  };

  const datasetId = createMemo(() => params.id);

  return (
    <DatasetContext.Provider
      value={{
        selectDataset,
        dataset: dataset,
        datasetId: datasetId,
      }}
    >
      {props.children}
    </DatasetContext.Provider>
  );
};
