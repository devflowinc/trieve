import { For, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { MenuItem } from "terracotta";
import { FaSolidCheck } from "solid-icons/fa";
import { DatasetAndUsageDTO } from "../../utils/apiTypes";

interface DatasetSelectionListProps {
  onSelect: () => void;
  datasets: DatasetAndUsageDTO[];
}
export const DatasetSelectionList = (props: DatasetSelectionListProps) => {
  const userContext = useContext(UserContext);
  const $currentDataset = userContext.currentDataset;
  return (
    <For each={props.datasets.slice(0, 500)}>
      {(datasetItem) => {
        return (
          <MenuItem
            as="button"
            classList={{
              "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white hover:bg-neutral-300 hover:dark:bg-neutral-700":
                true,
              "bg-neutral-300 dark:bg-neutral-700":
                datasetItem.dataset.id === $currentDataset?.()?.dataset.id,
            }}
            onClick={(e: Event) => {
              e.preventDefault();
              e.stopPropagation();
              userContext.setCurrentDataset(datasetItem);
              props.onSelect();
            }}
          >
            <div class="break-all px-1 text-left text-sm">
              {datasetItem.dataset.name}
            </div>
            <Show
              when={datasetItem.dataset.id == $currentDataset?.()?.dataset.id}
            >
              <span>
                <FaSolidCheck class="text-sm" />
              </span>
            </Show>
          </MenuItem>
        );
      }}
    </For>
  );
};
