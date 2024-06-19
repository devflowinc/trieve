import { BiRegularSearch, BiRegularX } from "solid-icons/bi";
import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { FaSolidCheck } from "solid-icons/fa";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { FilterModal } from "./FilterModal";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { SearchStore } from "../hooks/useSearch";

const SearchForm = (props: { search: SearchStore; groupID?: string }) => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const $envs = datasetAndUserContext.clientConfig;

  const [searchTypes, setSearchTypes] = createSignal([
    { name: "FullText", isSelected: false, route: "fulltext" },
    { name: "Semantic", isSelected: true, route: "semantic" },
    { name: "Hybrid", isSelected: false, route: "hybrid" },
    {
      name: "AutoComplete Semantic",
      isSelected: false,
      route: "autocomplete-semantic",
    },
    {
      name: "AutoComplete FullText",
      isSelected: false,
      route: "autocomplete-fulltext",
    },
  ]);

  createEffect(() => {
    props.search.setSearch(
      "searchType",
      searchTypes().find((type) => type.isSelected)?.route ?? "hybrid",
    );
  });

  createEffect(() => {
    $envs().CREATE_CHUNK_FEATURE?.valueOf();
  });

  const resetParams = () => {
    props.search.setSearch("scoreThreshold", 0.0);
    props.search.setSearch("extendResults", false);
    props.search.setSearch("slimChunks", false);
    props.search.setSearch("recencyBias", 0.0);
    props.search.setSearch("pageSize", 10);
    props.search.setSearch("getTotalPages", true);
    props.search.setSearch("highlightResults", true);
    props.search.setSearch("highlightDelimiters", ["?", ".", "!"]);
    props.search.setSearch("highlightMaxLength", 8);
    props.search.setSearch("highlightMaxNum", 3);
    props.search.setSearch("highlightWindow", 0);
  };

  return (
    <>
      <div class="w-full">
        <form class="w-full space-y-4 dark:text-white">
          <div class="relative flex">
            <div
              classList={{
                "flex w-full justify-center space-x-2 rounded-md bg-neutral-100 px-4 py-1 pr-[10px] dark:bg-neutral-700":
                  true,
              }}
            >
              <BiRegularSearch class="mt-1 h-6 w-6 fill-current" />
              <textarea
                id="search-query-textarea"
                placeholder="Search for chunks..."
                classList={{
                  "scrollbar-track-rounded-md scrollbar-thumb-rounded-md mr-2 h-fit max-h-[240px] w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600":
                    true,
                }}
                value={props.search.state.query}
                onInput={(e) => {
                  props.search.setSearch("query", e.currentTarget.value);
                }}
                onKeyDown={(e) => {
                  if (
                    ((e.ctrlKey || e.metaKey) && e.key === "Enter") ||
                    (!e.shiftKey && e.key === "Enter")
                  ) {
                    window.dispatchEvent(new Event("triggerSearch"));
                  }
                }}
                placeholder="Search for chunks..."
                rows={props.search.state.query.split("\n").length}
              />
              <Show when={props.search.state.query}>
                <button
                  classList={{
                    "pt-[2px]": !!props.search.state.query,
                  }}
                  onClick={(e) => {
                    e.preventDefault();
                    props.search.setSearch("query", "");
                  }}
                >
                  <BiRegularX class="h-7 w-7 fill-current" />
                </button>
              </Show>
              <Show when={props.search.state.query}>
                <button
                  classList={{
                    "border-l border-neutral-600 pl-[10px] dark:border-neutral-200":
                      !!props.search.state.query,
                  }}
                  type="submit"
                >
                  <BiRegularSearch class="mt-1 h-6 w-6 fill-current" />
                </button>
              </Show>
            </div>
          </div>
          <div class="flex space-x-3">
            <Popover defaultOpen={false} class="relative">
              {({ isOpen, setState }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle filters"
                    type="button"
                    class="flex items-center space-x-1 pb-1 text-sm"
                  >
                    <span>Filters</span>
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
                      tabIndex={0}
                      unmount={false}
                      class="absolute z-10 mt-2 h-fit w-fit rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-700"
                    >
                      <FilterModal
                        showFilterModal={isOpen}
                        setShowFilterModal={setState}
                      />
                    </PopoverPanel>
                  </Show>
                </>
              )}
            </Popover>
            <Popover defaultOpen={false} class="relative">
              {({ isOpen, setState }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle filters"
                    type="button"
                    class="flex items-center space-x-1 pb-1 text-sm"
                  >
                    <span>
                      Type:{" "}
                      {searchTypes().find((type) => type.isSelected)?.name ??
                        "Hybrid"}
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
                      class="absolute z-10 mt-2 h-fit w-[180px] rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                    >
                      <Menu class="ml-1 space-y-1">
                        <For each={searchTypes()}>
                          {(option) => {
                            if (props.groupID && option.route === "hybrid") {
                              return <></>;
                            }

                            const onClick = (e: Event) => {
                              e.preventDefault();
                              e.stopPropagation();
                              setSearchTypes((prev) => {
                                return prev.map((item) => {
                                  if (item.name === option.name) {
                                    return { ...item, isSelected: true };
                                  } else {
                                    return { ...item, isSelected: false };
                                  }
                                });
                              });
                              setState(true);
                            };
                            return (
                              <MenuItem
                                as="button"
                                classList={{
                                  "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white":
                                    true,
                                  "bg-neutral-300 dark:bg-neutral-900":
                                    option.isSelected,
                                }}
                                onClick={onClick}
                              >
                                <div class="flex flex-row justify-start space-x-2">
                                  <span class="text-left">{option.name}</span>
                                </div>
                                {option.isSelected && (
                                  <span>
                                    <FaSolidCheck class="fill-current text-xl" />
                                  </span>
                                )}
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
            <Popover defaultOpen={false} class="relative">
              {({ isOpen }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle options"
                    type="button"
                    class="flex items-center space-x-1 pb-1 text-sm"
                  >
                    <span>Options</span>
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
                      tabIndex={0}
                      class="absolute z-10 mt-2 h-fit w-[480px] rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-700"
                    >
                      <div class="items flex flex-col space-y-1">
                        <div class="mt-1">
                          <button
                            class="rounded-md border border-neutral-400 bg-neutral-100 px-2 py-1 dark:border-neutral-900 dark:bg-neutral-800"
                            onClick={(e) => {
                              e.preventDefault();
                              resetParams();
                            }}
                          >
                            Reset
                          </button>
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Score Threshold (0.0 to 1.0):</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            min="0.0"
                            max="1.0"
                            step="0.00001"
                            value={props.search.state.scoreThreshold}
                            onInput={(e) => {
                              props.search.setSearch(
                                "scoreThreshold",
                                parseFloat(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                        <Show
                          when={
                            searchTypes().find((type) => type.isSelected)
                              ?.route === "autocomplete-semantic" ||
                            searchTypes().find((type) => type.isSelected)
                              ?.route === "autocomplete-fulltext"
                          }
                        >
                          <div class="flex items-center justify-between space-x-2 p-1">
                            <label>Extend Results (autocomplete only):</label>
                            <input
                              class="h-4 w-4"
                              type="checkbox"
                              checked={props.search.state.extendResults}
                              onChange={(e) => {
                                props.search.setSearch(
                                  "extendResults",
                                  e.target.checked,
                                );
                              }}
                            />
                          </div>
                        </Show>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Slim Chunks (Latency Improvement):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.search.state.slimChunks}
                            onChange={(e) => {
                              props.search.setSearch(
                                "slimChunks",
                                e.target.checked,
                              );
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Recency Bias (0.0 to 1.0):</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            min="0.0"
                            max="1.0"
                            step="0.1"
                            value={props.search.state.recencyBias}
                            onInput={(e) => {
                              props.search.setSearch(
                                "recencyBias",
                                parseFloat(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Page Size:</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.search.state.pageSize}
                            onInput={(e) => {
                              props.search.setSearch(
                                "pageSize",
                                parseInt(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Get Total Pages (Latency Penalty):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.search.state.getTotalPages}
                            onChange={(e) => {
                              props.search.setSearch(
                                "getTotalPages",
                                e.target.checked,
                              );
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Highlight Results (Latency Penalty):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.search.state.highlightResults}
                            onChange={(e) => {
                              props.search.setSearch(
                                "highlightResults",
                                e.target.checked,
                              );
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Delimiters (Comma Separated):</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="text"
                            value={props.search.state.highlightDelimiters.join(
                              ",",
                            )}
                            onInput={(e) => {
                              if (e.currentTarget.value === " ") {
                                props.search.setSearch("highlightDelimiters", [
                                  " ",
                                ]);
                              }

                              props.search.setSearch(
                                "highlightDelimiters",
                                e.currentTarget.value.split(","),
                              );
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Max Length:</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.search.state.highlightMaxLength}
                            onInput={(e) => {
                              props.search.setSearch(
                                "highlightMaxLength",
                                parseInt(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Max Num:</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.search.state.highlightMaxNum}
                            onInput={(e) => {
                              props.search.setSearch(
                                "highlightMaxNum",
                                parseInt(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Window:</label>
                          <input
                            class="w-16 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.search.state.highlightWindow}
                            onInput={(e) => {
                              props.search.setSearch(
                                "highlightWindow",
                                parseInt(e.currentTarget.value),
                              );
                            }}
                          />
                        </div>
                      </div>
                    </PopoverPanel>
                  </Show>
                </>
              )}
            </Popover>
            <Show when={!props.groupID}>
              <div class="flex-1" />
              <div class="flex items-center space-x-2 justify-self-center">
                <label class="text-sm">Group Search</label>
                <input
                  class="h-4 w-4"
                  type="checkbox"
                  checked={props.search.state.groupUniqueSearch}
                  onChange={(e) => {
                    props.search.setSearch(
                      "groupUniqueSearch",
                      e.target.checked,
                    );
                  }}
                />
              </div>
            </Show>
          </div>
        </form>
      </div>
    </>
  );
};

export default SearchForm;
