import { persistentAtom } from "@nanostores/persistent";
import { atom } from "nanostores";
import type { DatasetAndUsageDTO } from "../../utils/apiTypes";
import { isDatasetAndUsageDTO } from "../../utils/apiTypes";
import { currentOrganization } from "./organizationStore";

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const apiHost: string = import.meta.env.VITE_API_HOST;

const tryParse = (encoded: string) => {
  try {
    if (isDatasetAndUsageDTO(JSON.parse(encoded))) {
      return JSON.parse(encoded) as DatasetAndUsageDTO;
    } else {
      return null;
    }
  } catch (e) {
    return null;
  }
};

export const currentDataset = persistentAtom("dataset", null, {
  encode: (dataset) => {
    const params = new URL(window.location.href).searchParams;
    params.set("dataset", (dataset as DatasetAndUsageDTO).dataset.id);
    window.history.replaceState(
      {},
      "",
      `${window.location.pathname}?${params.toString()}`,
    );
    return JSON.stringify(dataset);
  },
  decode: tryParse,
});
export const datasetsAndUsagesStore = atom<DatasetAndUsageDTO[]>([]);

currentOrganization.subscribe((organization) => {
  const params = new URLSearchParams(window.location.search);
  if (organization) {
    void fetch(`${apiHost}/dataset/organization/${organization.id}`, {
      method: "GET",
      credentials: "include",
      headers: {
        "TR-Organization": organization.id,
      },
    }).then((res) => {
      if (res.ok) {
        void res
          .json()
          .then((data) => {
            if (data && Array.isArray(data)) {
              if (data.length === 0) {
                datasetsAndUsagesStore.set([]);
              }
              if (data.length > 0 && data.every(isDatasetAndUsageDTO)) {
                if (
                  currentDataset.get() === null ||
                  params.get("dataset") === null
                ) {
                  currentDataset.set(data[0]);
                } else if (params.get("dataset") !== null) {
                  const dataset = data.find(
                    (d) => d.dataset.id === params.get("dataset"),
                  );
                  if (dataset) {
                    currentDataset.set(dataset);
                  } else {
                    currentDataset.set(data[0]);
                  }
                }
                datasetsAndUsagesStore.set(data);
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
