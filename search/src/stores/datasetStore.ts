import { atom } from "nanostores";
import type { DatasetAndUsageDTO } from "../../utils/apiTypes";
import { isDatasetAndUsageDTO } from "../../utils/apiTypes";
import { currentOrganization } from "./organizationStore";

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const apiHost: string = import.meta.env.PUBLIC_API_HOST;

export const currentDataset = atom<DatasetAndUsageDTO | null>(null);
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
                currentDataset.set(data[0]);
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
