import {
  Accessor,
  JSX,
  createEffect,
  createContext,
  createSignal,
  useContext,
} from "solid-js";
import { Dataset } from "shared/types";
import { UserContext } from "./UserContext";
import { useParams } from "@solidjs/router";
import { createToast } from "../components/ShowToasts";

export interface DatasetStoreContextProps {
  children: JSX.Element;
}

export interface DatasetStore {
  dataset: Accessor<Dataset | null> | null;
}

export const DatasetContext = createContext<DatasetStore>({
  dataset: null,
});

export const DatasetContextWrapper = (props: DatasetStoreContextProps) => {
  const userContext = useContext(UserContext);
  const [dataset, setDataset] = createSignal<Dataset | null>(null);

  createEffect(() => {
    if (userContext?.user?.()) {
      const id = useParams().id;
      if (!id) return;

      if (!id || !id.match(/^[a-f0-9-]+$/)) {
        console.error("Invalid dataset id for fetch");
        return;
      }

      const api_host = import.meta.env.VITE_API_HOST as string;
      fetch(`${api_host}/dataset/${id}`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          "TR-Organization": userContext.selectedOrganizationId?.() as string,
        },
        credentials: "include",
      })
        .then((res) => res.json())
        .then((data) => {
          setDataset(data);
        })
        .catch(() => {
          createToast({
            title: "Error",
            type: "error",
            message: "Failed to fetch the dataset",
          });
        });
    }
  });

  const datasetStore: DatasetStore = {
    dataset,
  };

  return (
    <DatasetContext.Provider value={datasetStore}>
      {props.children}
    </DatasetContext.Provider>
  );
};
