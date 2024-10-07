import { useContext } from "solid-js";
import { DatasetAndUserContext } from "../components/Contexts/DatasetAndUserContext";

export const useCtrClickForChunk = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const registerClickForChunk = async (
    id: string,
    position: number,
    searchID: string,
    type?: string,
  ) => {
    const dataset = $dataset?.();
    if (!dataset) return;
    const data = await fetch(`${apiHost}/analytics/ctr`, {
      method: "PUT",
      body: JSON.stringify({
        clicked_chunk_id: id,
        position: position,
        ctr_type: type || "search",
        request_id: searchID,
      }),
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
    });
    console.log(data);
  };

  return {
    registerClickForChunk,
  };
};
