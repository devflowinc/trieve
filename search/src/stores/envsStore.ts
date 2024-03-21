import { persistentAtom } from "@nanostores/persistent";
import {
  ClientEnvsConfiguration,
  defaultClientEnvsConfiguration,
} from "../../utils/apiTypes";
import { currentDataset } from "./datasetStore";

const apiHost = window.API_HOST || import.meta.env.VITE_API_HOST as unknown as string;

const tryParse = (encoded: string) => {
  try {
    if (JSON.parse(encoded)) {
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
    void fetch(`${apiHost}/dataset/envs`, {
      method: "GET",
      credentials: "include",
      headers: {
        "TR-Dataset": dataset.dataset.id,
      },
    }).then((res) => {
      if (res.ok) {
        void res
          .json()
          .then((data) => {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
            clientConfig.set(data);
          })
          .catch((err) => {
            console.log(err);
          });
      }
    });
  }
});
