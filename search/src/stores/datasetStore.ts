import { persistentAtom } from "@nanostores/persistent";
import { atom } from "nanostores";
import type { DatasetAndUsageDTO } from "../../utils/apiTypes";
import { isDatasetAndUsageDTO } from "../../utils/apiTypes";
import { currentOrganization } from "./organizationStore";

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const apiHost: string = import.meta.env.PUBLIC_API_HOST;

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
  encode: JSON.stringify,
  decode: tryParse,
});
export const datasetsAndUsagesStore = atom<DatasetAndUsageDTO[]>([]);

currentOrganization.subscribe((organization) => {
  if (organization) {
    void fetch(`${apiHost}/dataset/organization/${organization.id}`, {
      method: "GET",
      credentials: "include",
      headers: {
        "AF-Organization": organization.id,
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
                if (currentDataset.get() === null) {
                  currentDataset.set(data[0]);
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
