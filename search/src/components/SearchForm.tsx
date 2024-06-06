import { BiRegularSearch, BiRegularX } from "solid-icons/bi";
import { useNavigate } from "@solidjs/router";
import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
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

const SearchForm = (props: {
  query?: string;
  searchType: string;
  extendResults?: boolean;
  groupUniqueSearch?: boolean;
  slimChunks?: boolean;
  pageSize?: number;
  getTotalPages?: boolean;
  highlightResults?: boolean;
  highlightDelimiters?: string[];
  highlightMaxLength?: number;
  highlightMaxNum?: number;
  recencyBias?: number;
  groupID?: string;
}) => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const $envs = datasetAndUserContext.clientConfig;
  const navigate = useNavigate();

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
  const [textareaInput, setTextareaInput] = createSignal("");
  const [typewriterEffect, setTypewriterEffect] = createSignal("");
  const [textareaFocused, setTextareaFocused] = createSignal(false);
  const [groupUniqueSearch, setGroupUniqueSearch] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.groupUniqueSearch ?? false,
  );
  // eslint-disable-next-line solid/reactivity
  const [slimChunks, setSlimChunks] = createSignal(props.slimChunks ?? false);
  const [pageSize, setPageSize] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.pageSize ?? 10,
  );
  const [getTotalPages, setGetTotalPages] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.getTotalPages ?? false,
  );
  const [highlightResults, setHighlightResults] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.highlightResults ?? true,
  );
  const [highlightDelimiters, setHighlightDelimiters] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.highlightDelimiters ?? ["?", ".", "!"],
  );
  const [highlightMaxLength, setHighlightMaxLength] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.highlightMaxLength ?? 8,
  );
  const [highlightMaxNum, setHighlightMaxNum] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.highlightMaxNum ?? 3,
  );
  const [recencyBias, setRecencyBias] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.recencyBias ?? 0.0,
  );
  const [extendResults, setExtendResults] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.extendResults ?? false,
  );

  const resizeTextarea = (textarea: HTMLTextAreaElement | null) => {
    if (!textarea) return;

    textarea.style.height = `${textarea.scrollHeight}px`;
  };

  const onSubmit = (e: Event) => {
    e.preventDefault();
    const textAreaValue = textareaInput();
    if (!textAreaValue) return;

    const searchQuery = encodeURIComponent(
      textAreaValue.length > 3800
        ? textAreaValue.slice(0, 3800)
        : textAreaValue,
    );

    const searchTypeRoute =
      searchTypes().find((type) => type.isSelected)?.route ?? "hybrid";
    const searchTypeUrlParam = searchTypeRoute
      ? `&searchType=${searchTypeRoute}`
      : "";
    let extendResultsUrlParam = "";

    if (searchTypeRoute.includes("autocomplete")) {
      extendResultsUrlParam = extendResults() ? "&extendResults=true" : "";
    }
    const groupUniqueUrlParam = groupUniqueSearch() ? "&groupUnique=true" : "";
    const slimChunksUrlParam = slimChunks() ? "&slimChunks=true" : "";
    const recencyBiasUrlParam = recencyBias()
      ? `&recencyBias=${recencyBias()}`
      : "";
    const pageSizeUrlParam = pageSize() ? `&pageSize=${pageSize()}` : "";
    const getTotalPagesUrlParam = getTotalPages() ? "&getTotalPages=true" : "";
    const highlightResultsUrlParam = highlightResults()
      ? "&highlightResults=true"
      : "";
    const highlightDelimitersUrlParam = highlightDelimiters().length
      ? `&highlightDelimiters=${highlightDelimiters().join(",")}`
      : "";
    const highlightMaxLengthUrlParam = highlightMaxLength()
      ? `&highlightMaxLength=${highlightMaxLength()}`
      : "";
    const highlightMaxNumUrlParam = highlightMaxNum()
      ? `&highlightMaxNum=${highlightMaxNum()}`
      : "";

    const sharedUrlParams =
      searchTypeUrlParam +
      extendResultsUrlParam +
      slimChunksUrlParam +
      recencyBiasUrlParam +
      pageSizeUrlParam +
      getTotalPagesUrlParam +
      highlightDelimitersUrlParam +
      highlightResultsUrlParam +
      highlightMaxLengthUrlParam +
      highlightMaxNumUrlParam;

    const urlToNavigateTo = props.groupID
      ? `/group/${props.groupID}?q=${searchQuery}` + sharedUrlParams
      : `/search?q=${searchQuery}` + groupUniqueUrlParam + sharedUrlParams;

    navigate(urlToNavigateTo);
  };

  createEffect(() => {
    setTextareaInput(props.query ?? "");

    setSearchTypes((prev) => {
      return prev.map((item) => {
        if (props.searchType == item.route) {
          return { ...item, isSelected: true };
        } else {
          return { ...item, isSelected: false };
        }
      });
    });

    setTimeout(() => {
      resizeTextarea(document.querySelector("#search-query-textarea"));
    }, 5);
  });

  createEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    textareaVal();

    resizeTextarea(document.querySelector("#search-query-textarea"));
  });

  createEffect(() => {
    const shouldNotRun = textareaInput() || textareaFocused();

    if (shouldNotRun) {
      return;
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    const textArray: string[] = $envs().SEARCH_QUERIES?.split(",") ?? [];

    const typingSpeed = 50;
    const deleteSpeed = 30;

    let currentTextIndex = 0;
    let currentCharIndex = 0;
    let isDeleting = false;

    let timeoutRefOne: number;
    let timeoutRefTwo: number;
    let timeoutRefThree: number;

    const typeText = () => {
      const currentText = textArray[currentTextIndex];

      if (isDeleting) {
        setTypewriterEffect(currentText.substring(0, currentCharIndex - 1));
        currentCharIndex--;
      } else {
        setTypewriterEffect(currentText.substring(0, currentCharIndex + 1));
        currentCharIndex++;
      }

      if (!isDeleting && currentCharIndex === currentText.length) {
        isDeleting = true;
        timeoutRefOne = setTimeout(typeText, 1000) as unknown as number;
      } else if (isDeleting && currentCharIndex === 0) {
        isDeleting = false;
        currentTextIndex = (currentTextIndex + 1) % textArray.length;
        timeoutRefTwo = setTimeout(typeText, typingSpeed) as unknown as number;
      } else {
        const speed = isDeleting ? deleteSpeed : typingSpeed;
        timeoutRefThree = setTimeout(typeText, speed) as unknown as number;
      }
    };

    typeText();

    onCleanup(() => {
      clearTimeout(timeoutRefOne);
      clearTimeout(timeoutRefTwo);
      clearTimeout(timeoutRefThree);
    });
  });

  createEffect(() => {
    $envs().CREATE_CHUNK_FEATURE?.valueOf();
  });

  const textareaVal = createMemo(() => {
    const textareaInputVal = textareaInput();
    const textareaFocusedVal = textareaFocused();
    const typewriterEffectVal = typewriterEffect();
    const textareaVal =
      textareaInputVal ||
      (textareaFocusedVal ? textareaInputVal : typewriterEffectVal);

    return textareaVal;
  });

  return (
    <>
      <div class="w-full">
        <form class="w-full space-y-4 dark:text-white" onSubmit={onSubmit}>
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
                classList={{
                  "scrollbar-track-rounded-md scrollbar-thumb-rounded-md mr-2 h-fit max-h-[240px] w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600":
                    true,
                  "text-neutral-600": !textareaInput() && !textareaFocused(),
                }}
                onFocus={() => setTextareaFocused(true)}
                onBlur={() => setTextareaFocused(false)}
                value={textareaVal()}
                onInput={(e) => {
                  setTextareaInput(e.target.value);
                }}
                onKeyDown={(e) => {
                  if (
                    ((e.ctrlKey || e.metaKey) && e.key === "Enter") ||
                    (!e.shiftKey && e.key === "Enter")
                  ) {
                    window.dispatchEvent(new Event("triggerSearch"));

                    onSubmit(e);
                  }
                }}
                rows="1"
              />
              <Show when={textareaInput()}>
                <button
                  classList={{
                    "pt-[2px]": !!props.query,
                  }}
                  onClick={(e) => {
                    e.preventDefault();
                    setTextareaInput("");
                  }}
                >
                  <BiRegularX class="h-7 w-7 fill-current" />
                </button>
              </Show>
              <Show when={props.query}>
                <button
                  classList={{
                    "border-l border-neutral-600 pl-[10px] dark:border-neutral-200":
                      !!textareaInput(),
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
                              onSubmit(e);
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
              {({ isOpen, setState }) => (
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
                      class="absolute z-10 mt-2 h-fit w-[480px] rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-700"
                    >
                      <div class="items flex flex-col space-y-1">
                        <div class="mt-1">
                          <button
                            class="rounded-md border border-neutral-400 bg-neutral-100 px-2 py-1 dark:border-neutral-900 dark:bg-neutral-800"
                            onClick={(e) => {
                              e.preventDefault();
                              setSlimChunks(false);
                              setPageSize(10);
                              setGetTotalPages(false);
                              setHighlightResults(true);
                              setHighlightDelimiters(["?", ".", "!"]);
                              setRecencyBias(0.0);
                              setState(false);
                              onSubmit(e);
                            }}
                          >
                            Reset
                          </button>
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
                              checked={props.extendResults}
                              onChange={(e) => {
                                if (e.target.checked) {
                                  setExtendResults(true);
                                } else {
                                  setExtendResults(false);
                                }

                                onSubmit(e);
                              }}
                            />
                          </div>
                        </Show>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Slim Chunks (Latency Improvement):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.slimChunks}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setSlimChunks(true);
                              } else {
                                setSlimChunks(false);
                              }

                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Recency Bias (0.0 to 1.0):</label>
                          <input
                            class="w-12 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            min="0.0"
                            max="1.0"
                            step="0.1"
                            value={props.recencyBias}
                            onInput={(e) => {
                              setRecencyBias(parseFloat(e.currentTarget.value));
                            }}
                            onBlur={(e) => {
                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Page Size:</label>
                          <input
                            class="w-12 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.pageSize}
                            onInput={(e) => {
                              setPageSize(parseInt(e.currentTarget.value));
                            }}
                            onBlur={(e) => {
                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Get Total Pages (Latency Penalty):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.getTotalPages}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setGetTotalPages(true);
                              } else {
                                setGetTotalPages(false);
                              }

                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="flex items-center justify-between space-x-2 p-1">
                          <label>Highlight Results (Latency Penalty):</label>
                          <input
                            class="h-4 w-4"
                            type="checkbox"
                            checked={props.highlightResults}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setHighlightResults(true);
                              } else {
                                setHighlightResults(false);
                              }

                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Delimiters (Comma Separated):</label>
                          <input
                            class="w-12 rounded border border-neutral-400 p-0.5 text-black"
                            type="text"
                            value={highlightDelimiters().join(",")}
                            onInput={(e) => {
                              if (e.currentTarget.value === " ") {
                                setHighlightDelimiters([" "]);
                              }

                              setHighlightDelimiters(
                                e.currentTarget.value.split(","),
                              );
                            }}
                            onBlur={(e) => {
                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Max Length:</label>
                          <input
                            class="w-12 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.highlightMaxLength}
                            onInput={(e) => {
                              setHighlightMaxLength(
                                parseInt(e.currentTarget.value),
                              );
                            }}
                            onBlur={(e) => {
                              onSubmit(e);
                            }}
                          />
                        </div>
                        <div class="items flex justify-between space-x-2 p-1">
                          <label>Highlight Max Num:</label>
                          <input
                            class="w-12 rounded border border-neutral-400 p-0.5 text-black"
                            type="number"
                            value={props.highlightMaxNum}
                            onInput={(e) => {
                              setHighlightMaxNum(
                                parseInt(e.currentTarget.value),
                              );
                            }}
                            onBlur={(e) => {
                              onSubmit(e);
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
                  checked={props.groupUniqueSearch}
                  onChange={(e) => {
                    if (e.target.checked) {
                      setGroupUniqueSearch(true);
                    } else {
                      setGroupUniqueSearch(false);
                    }

                    onSubmit(e);
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
