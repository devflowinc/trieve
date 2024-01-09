import { FaSolidDatabase } from "solid-icons/fa";
import { Show, For, createSignal } from "solid-js";
import { currentDataset, datasetsAndUsagesStore } from "../stores/datasetStore";
import { useStore } from "@nanostores/solid";
import { BsChevronExpand } from "solid-icons/bs";
import { Select, SelectOption } from "solid-headless";
import type { DatasetAndUsageDTO } from "../../utils/apiTypes";

export const DatasetSelectBox = () => {
  const [isDatasetSelectOpen, setIsDatasetSelectOpen] = createSignal(false);
  const $datasetsAndUsages = useStore(datasetsAndUsagesStore);
  const $currentDataset = useStore(currentDataset);

  return (
    <div>
      <Show when={$datasetsAndUsages().length > 0}>
        <div class="flex items-center space-x-4">
          <FaSolidDatabase class="h-5 w-5 text-gray-800" />
          <div class="w-full">
            <button
              onClick={() => {
                setIsDatasetSelectOpen((prev) => !prev);
              }}
              type="button"
              class="relative w-full cursor-default rounded-md bg-white py-1.5 pl-3 pr-10 text-left text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 focus:outline-none focus:ring-2 focus:ring-indigo-600 sm:text-sm sm:leading-6"
              aria-haspopup="listbox"
              aria-expanded="true"
              aria-labelledby="listbox-label"
            >
              <span class="block truncate">
                {$currentDataset()?.dataset.name}
              </span>
              <BsChevronExpand
                class="absolute inset-y-0 right-0 mr-2 mt-2 h-5 w-5 text-gray-400"
                aria-hidden="true"
              />
            </button>
            <Show when={isDatasetSelectOpen()}>
              <Select
                toggleable
                value={$currentDataset()}
                onchange={(dataset: DatasetAndUsageDTO | null) => {
                  currentDataset.set(dataset);
                }}
                class="absolute z-10 w-[500px] border border-magenta bg-white"
              >
                <For each={$datasetsAndUsages()}>
                  {(item) => (
                    <SelectOption value={item}>
                      {({ isActive, isSelected }) => (
                        <div
                          classList={{
                            "flex items-center space-x-2": true,
                            "bg-green-100 dark:bg-neutral-700": isActive(),
                            "bg-red-200 dark:bg-neutral-600": isSelected(),
                          }}
                        >
                          {item.dataset.name}
                        </div>
                      )}
                    </SelectOption>
                  )}
                </For>
              </Select>
            </Show>
          </div>
        </div>
      </Show>
      <Show when={$datasetsAndUsages().length === 0}>
        <div class="flex items-center space-x-4">
          You have no datasets silly baka
        </div>
      </Show>
    </div>
  );
};
