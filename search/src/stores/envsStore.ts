import { atom } from "nanostores";
import {
  isClientEnvsConfiguration,
  type ClientEnvsConfiguration,
} from "../../utils/apiTypes";
import { currentDataset } from "./datasetStore";

const api_host = import.meta.env.PUBLIC_API_HOST as unknown as string;

export const clientConfig = atom<ClientEnvsConfiguration | null>(null);

currentDataset.subscribe((dataset) => {
  if (dataset) {
    void fetch(`${api_host}/dataset/envs`, {
      method: "GET",
      credentials: "include",
      headers: {
        "AF-Dataset": dataset.dataset.id,
      },
    }).then((res) => {
      if (res.ok) {
        void res
          .json()
          .then((data) => {
            if (data && Array.isArray(data)) {
              if (data.length === 0) {
                clientConfig.set(null);
              }
              if (data.length > 0 && data.every(isClientEnvsConfiguration)) {
                clientConfig.set(data[0]);
              }
            }
          })
          .catch((err) => {
            console.log(err);
          });
      }
    });
  }
});
