/* eslint-disable @typescript-eslint/unbound-method */
import { FaSolidCheck } from "solid-icons/fa";
import {
  Show,
  For,
  createMemo,
  useContext,
  Switch,
  Match,
  createSignal,
  createEffect,
} from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { DatasetAndUsageDTO } from "../utils/apiTypes";
import createFuzzySearch from "@nozbe/microfuzz";

export const DatasetSelectBox = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const [datasetSearchQuery, setDatasetSearchQuery] = createSignal("");

  const $datasetsAndUsages = datasetAndUserContext.datasetsAndUsages;
  const $currentDataset = datasetAndUserContext.currentDataset;

  const datasetList = createMemo(() => $datasetsAndUsages?.());
  const [datasetSearchResults, setDatasetSearchResults] = createSignal<
    DatasetAndUsageDTO[]
  >([]);

  createEffect(() => {
    const datasetListOrEmpty = datasetList() ?? [];
    if (datasetSearchQuery() === "") {
      setDatasetSearchResults(datasetListOrEmpty);
    } else {
      const fuzzy = createFuzzySearch(datasetListOrEmpty, {
        getText: (item: DatasetAndUsageDTO) => {
          return [item.dataset.name];
        },
      });
      const results = fuzzy(datasetSearchQuery() ?? "");
      setDatasetSearchResults(results.map((result) => result.item));
    }
  });

  return (
    <Show when={$datasetsAndUsages?.().length != 0}>
      <Popover defaultOpen={false} class="relative">
        {({ isOpen, setState }) => (
          <>
            <PopoverButton
              aria-label="Select Dataset"
              type="button"
              class="flex min-w-fit items-center space-x-1 pb-1 text-sm"
            >
              <span class="line-clamp-1 min-w-fit text-left text-sm">
                {$currentDataset?.()?.dataset.name}
              </span>
              <Switch>
                <Match when={isOpen()}>
                  <FiChevronUp class="h-3.5 w-3.5" />
                </Match>
                <Match when={!isOpen()}>
                  <FiChevronDown class="h-3.5 w-3.5" />
                </Match>
              </Switch>
            </PopoverButton>
            <Show when={isOpen()}>
              <PopoverPanel
                unmount={false}
                class="absolute right-0 z-10 mt-2 h-fit w-[180px] rounded-md border bg-white dark:bg-neutral-800"
              >
                <Menu class="mx-0.5 max-h-[70vh] space-y-0.5 overflow-y-auto overflow-x-hidden p-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-600 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-400">
                  <Show when={$datasetsAndUsages?.().length ?? 0 > 5}>
                    <input
                      placeholder="Search datasets..."
                      class="mb-2 flex w-full items-center justify-between rounded bg-neutral-300 p-1 text-sm text-black outline-none dark:bg-neutral-700 dark:hover:text-white dark:focus:text-white"
                      onInput={(e) => {
                        setDatasetSearchQuery(e.target.value);
                      }}
                      value={datasetSearchQuery()}
                    />
                  </Show>

                  <For each={datasetSearchResults().slice(0, 500)}>
                    {(datasetItem) => {
                      return (
                        <MenuItem
                          as="button"
                          classList={{
                            "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white hover:bg-neutral-300 hover:dark:bg-neutral-700":
                              true,
                            "bg-neutral-300 dark:bg-neutral-700":
                              datasetItem.dataset.id ===
                              $currentDataset?.()?.dataset.id,
                          }}
                          onClick={(e: Event) => {
                            e.preventDefault();
                            e.stopPropagation();
                            datasetAndUserContext.setCurrentDataset(
                              datasetItem,
                            );
                            setState(false);
                          }}
                        >
                          <div class="break-all px-1 text-left text-sm">
                            {datasetItem.dataset.name}
                          </div>
                          <Show
                            when={
                              datasetItem.dataset.id ==
                              $currentDataset?.()?.dataset.id
                            }
                          >
                            <span>
                              <FaSolidCheck class="text-sm" />
                            </span>
                          </Show>
                        </MenuItem>
                      );
                    }}
                  </For>
                </Menu>
              </PopoverPanel>
            </Show>
          </>
        )}
      </Popover>
    </Show>
  );
};
