import { createMemo, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { Select } from "shared/ui";
import { DatasetContext } from "../contexts/DatasetContext";
import { FiDatabase } from "solid-icons/fi";

export const NavbarDatasetSelector = () => {
  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);
  const datasetIds = createMemo(
    () => userContext.orgDatasets()?.map((dataset) => dataset.dataset.id),
  );

  const datasetNameFromId = (id: string) => {
    const dataset = userContext
      .orgDatasets()
      ?.find((dataset) => dataset.dataset.id === id);
    if (dataset) {
      return dataset.dataset.name;
    }
  };

  return (
    <div>
      <Show when={datasetIds()}>
        {(datasets) => (
          <Select
            class="bg-white"
            onSelected={datasetContext.selectDataset}
            display={(id) => id}
            displayElement={(id) => (
              <div class="flex items-center gap-2">
                <FiDatabase class="text-neutral-400" />
                <div class="text-sm">{datasetNameFromId(id)}</div>
              </div>
            )}
            selected={datasetContext.datasetId}
            options={datasets()}
          />
        )}
      </Show>
    </div>
  );
};
