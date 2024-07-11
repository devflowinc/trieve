import { createResource, useContext } from "solid-js";
import { DatasetAndUserContext } from "../components/Contexts/DatasetAndUserContext";
import { DatasetDTO } from "../utils/apiTypes";

const apiHost = import.meta.env.VITE_API_HOST as string;

export const useDatasetServerConfig = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const [data] = createResource(
    () => datasetAndUserContext?.currentDataset?.(),
    async (d) => {
      const response = await fetch(`${apiHost}/dataset/${d.dataset.id}`, {
        credentials: "include",
        headers: {
          "X-API-version": "2.0",
          "TR-Dataset": d.dataset.id,
        },
      });

      if (!response.ok) {
        throw new Error("Failed to fetch dataset");
      }

      const dataset = (await response.json()) as DatasetDTO;
      return dataset.server_configuration;
    },
  );

  return data;
};
