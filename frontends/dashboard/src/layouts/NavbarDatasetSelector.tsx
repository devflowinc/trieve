import { createMemo, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { Select } from "shared/ui";
import { DatasetContext } from "../contexts/DatasetContext";
import { FiDatabase } from "solid-icons/fi";

export const NavbarDatasetSelector = () => {
  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);

  const datasetIds = createMemo(() => {
    const orgDatasets = userContext.orgDatasets();
    if (Array.isArray(orgDatasets)) {
      return orgDatasets.map((dataset) => dataset.dataset.id);
    }

    return undefined;
  });

  const datasetNameFromId = (id: string) => {
    const dataset = userContext
      .orgDatasets()
      ?.find((dataset) => dataset.dataset.id === id);
    if (dataset) {
      return dataset.dataset.name;
    }

    return "Click to Select Dataset";
  };

  return (
    <div>
      <Show when={datasetIds()}>
        {(datasets) => (
          <Select
            class={`bg-white ${
              !datasetContext.datasetId() ? "text-neutral-600" : "text-black"
            }`}
            onSelected={datasetContext.selectDataset}
            display={(id) => datasetNameFromId(id)}
            displayElement={(id) => (
              <div class="flex items-center gap-2">
                <FiDatabase />
                <div class="text-sm">{datasetNameFromId(id)}</div>
              </div>
            )}
            selected={datasetContext.datasetId()}
            options={datasets()}
          />
        )}
      </Show>
    </div>
  );
};
