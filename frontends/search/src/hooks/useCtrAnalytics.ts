import { useContext } from "solid-js";
import { DatasetAndUserContext } from "../components/Contexts/DatasetAndUserContext";

export const useCtrClickForChunk = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const registerClickForChunk = async ({
    id,
    position,
    searchID,
    eventType,
  }: {
    id: string;
    position?: number;
    searchID?: string;
    eventType: string;
  }) => {
    let body = {};
    const dataset = $dataset?.();
    if (!dataset) return;

    switch (eventType) {
      case "click":
        body = {
          event_name: "Click",
          clicked_items: {
            chunk_id: id,
            position: (position || 0) + 1,
          },
          request_id: searchID || null,
          event_type: "click",
        };
        break;
      case "view":
        body = {
          event_name: "View",
          items: [id],
          request_id: searchID || null,
          event_type: "view",
        };
        break;
      case "add_to_cart":
        body = {
          event_name: "Added to Cart",
          items: [id],
          request_id: searchID || null,
          event_type: "add_to_cart",
        };
        break;
      case "purchase":
        body = {
          event_name: "Purchase",
          items: [id],
          request_id: searchID || null,
          event_type: "purchase",
        };
        break;
    }

    const data = await fetch(`${apiHost}/analytics/events`, {
      method: "PUT",
      body: JSON.stringify(body),
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
