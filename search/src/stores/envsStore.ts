import { persistentAtom } from "@nanostores/persistent";
import {
  isClientEnvsConfiguration,
  ClientEnvsConfiguration,
  defaultClientEnvsConfiguration,
} from "../../utils/apiTypes";
import { currentDataset } from "./datasetStore";

const api_host = import.meta.env.PUBLIC_API_HOST as unknown as string;

const tryParse = (encoded: string) => {
  try {
    if (isClientEnvsConfiguration(JSON.parse(encoded))) {
      return JSON.parse(encoded) as ClientEnvsConfiguration;
    } else {
      return defaultClientEnvsConfiguration;
    }
  } catch (e) {
    return defaultClientEnvsConfiguration;
  }
};

export const clientConfig = persistentAtom(
  "clientConfig",
  defaultClientEnvsConfiguration,
  {
    encode: JSON.stringify,
    decode: tryParse,
  },
);

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
