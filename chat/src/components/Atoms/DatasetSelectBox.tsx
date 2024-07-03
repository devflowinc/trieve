import {
  Show,
  useContext,
  createSignal,
  Switch,
  Match,
  createMemo,
  onMount,
} from "solid-js";
import { Menu, Popover, PopoverButton, PopoverPanel } from "terracotta";
import { UserContext } from "../contexts/UserContext";
import { DatasetAndUsageDTO } from "../../utils/apiTypes";
import createFuzzySearch from "@nozbe/microfuzz";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { DatasetSelectionList } from "./DatasetSelectionList";

export const DatasetSelectBox = () => {
  const userContext = useContext(UserContext);
  const $datasetsAndUsages = userContext.datasetsAndUsages;
  const $currentDataset = userContext.currentDataset;
  const [datasetSearchQuery, setDatasetSearchQuery] = createSignal("");
  let inputRef: HTMLInputElement | undefined;

  const searchResults = createMemo(() => {
    const datasetListOrEmpty = userContext.datasetsAndUsages?.() ?? [];
    if (datasetSearchQuery() === "") {
      return datasetListOrEmpty;
    }
    const fuzzy = createFuzzySearch(datasetListOrEmpty, {
      getText: (item: DatasetAndUsageDTO) => {
        return [item.dataset.name, item.dataset.id];
      },
    });
    const results = fuzzy(datasetSearchQuery());
    return results.map((res) => res.item);
  });

  const focusInput = () => {
    if (inputRef) {
      inputRef.focus();
    }
  };

  onMount(() => {
    focusInput();
  });

  return (
    <Show when={$datasetsAndUsages?.().length != 0}>
      <Popover defaultOpen={false} class="relative">
        {({ isOpen, setState }) => (
          <>
            <PopoverButton
              aria-label="Toggle filters"
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
                class="absolute bottom-5 left-0 z-10 mt-2 h-fit w-[180px] rounded-md border bg-white p-1 dark:bg-neutral-800"
              >
                <Menu class="mx-1 space-y-0.5">
                  <input
                    ref={inputRef}
                    placeholder="Search datasets..."
                    class="mb-2 flex w-full items-center justify-between rounded bg-neutral-300 p-1 text-sm text-black outline-none dark:bg-neutral-700 dark:hover:text-white dark:focus:text-white"
                    onInput={(e) => {
                      setDatasetSearchQuery(e.target.value);
                    }}
                    onFocusOut={(e) => {
                      if (
                        !e.relatedTarget ||
                        !(e.relatedTarget instanceof Node) ||
                        !e.currentTarget.contains(e.relatedTarget)
                      ) {
                        focusInput();
                      }
                    }}
                    value={datasetSearchQuery()}
                  />
                  <DatasetSelectionList
                    onSelect={() => {
                      setState(false);
                      setDatasetSearchQuery("");
                    }}
                    datasets={searchResults()}
                  />
                </Menu>
              </PopoverPanel>
            </Show>
          </>
        )}
      </Popover>
    </Show>
  );
};

