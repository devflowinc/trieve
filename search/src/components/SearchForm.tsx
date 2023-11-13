import { BiRegularSearch, BiRegularX } from "solid-icons/bi";
import { AiOutlineClockCircle } from "solid-icons/ai";
import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Combobox, ComboboxSection } from "./Atoms/ComboboxChecklist";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { FaSolidCheck } from "solid-icons/fa";
import type { Filters } from "./ResultsPage";
import { DatePicker } from "./Atoms/DatePicker";

const SearchForm = (props: {
  query?: string;
  filters: Filters;
  searchType: string;
  collectionID?: string;
}) => {
  const stringifiedComboboxSections = import.meta.env
    .PUBLIC_FILTER_ITEMS as string;

  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const comboboxSections: ComboboxSection[] = stringifiedComboboxSections
    ? JSON.parse(stringifiedComboboxSections)
    : [];
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const customComboBoxFilterVals: ComboboxSection[] = comboboxSections;

  const createEvidenceFeature =
    import.meta.env.PUBLIC_CREATE_EVIDENCE_FEATURE !== "off";
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
  const feelingSuffixes: string[] = (import.meta.env.PUBLIC_LUCKY_ITEMS || "")
    .split(",")
    .filter((item: string) => item);

  const [searchTypes, setSearchTypes] = createSignal([
    { name: "Full Text", isSelected: false, route: "fulltextsearch" },
    { name: "Semantic", isSelected: true, route: "search" },
    { name: "Hybrid Search", isSelected: false, route: "hybrid_search" },
  ]);
  // eslint-disable-next-line solid/reactivity
  const [textareaInput, setTextareaInput] = createSignal(props.query ?? "");
  const [typewriterEffect, setTypewriterEffect] = createSignal("");
  const [textareaFocused, setTextareaFocused] = createSignal(false);
  const [comboBoxSections, setComboBoxSections] = createSignal<
    ComboboxSection[]
  >(customComboBoxFilterVals);
  const [feelingLuckyText, setFeelingLuckyText] = createSignal(feelingSuffixes);
  const [feelingLuckySpinning, setFeelingLuckySpinning] = createSignal(false);
  const [searchQueriesFromStorage, setSearchQueriesFromStorage] = createSignal<
    string[]
  >([]);
  const [searchHistoryList, setSearchHistoryList] = createSignal<string[]>([]);
<<<<<<< HEAD
  const [showFilters, setShowFilters] = createSignal(false);
  const [timeRange, setTimeRange] = createSignal({
    start: props.filters.start,
    end: props.filters.end,
  });
=======
  const [semanticWeight, setSemanticWeight] = createSignal("0.5");
>>>>>>> 0c8dfaf (feature: UI for hybrid weights, bugfix: 0/1 vals for semantic/full-text weights)

  createEffect(() => {
    // get the previous searched queries from localStorage and set them into the state;
    const searchQueriesFromStorage = localStorage.getItem("searchQueries");

    if (searchQueriesFromStorage) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const parsedQueries: string[] = JSON.parse(searchQueriesFromStorage);
      setSearchQueriesFromStorage(parsedQueries);
    }
  });

  const updateSearchQueriesInStorage = (
    query: string,
    searchQueriesFromStorage: (string | null)[],
  ) => {
    if (searchQueriesFromStorage.includes(query)) {
      return;
    }
    const queriesArray = [
      ...searchQueriesFromStorage,
      query,
    ] as unknown as string[];
    setSearchQueriesFromStorage(queriesArray);
    localStorage.setItem("searchQueries", JSON.stringify(queriesArray));
  };

  const updateSearchHistory = (query: string, searchQueries: string[]) => {
    const storedQueries = searchQueries;
    if (!query) {
      return;
    }

    const filteredQueries = storedQueries.filter((storedQuery: string) => {
      return storedQuery.toLowerCase().startsWith(query.toLowerCase());
    });

    filteredQueries.sort((a: string, b: string) => {
      const aLower = a.toLowerCase();
      const bLower = b.toLowerCase();

      if (aLower < bLower) return -1;
      if (aLower > bLower) return 1;
      return 0;
    });

    setSearchHistoryList(filteredQueries.slice(0, 5));
  };

  const resizeTextarea = (
    textarea: HTMLTextAreaElement | null,
    avoidSetInput: boolean | undefined,
  ) => {
    if (!textarea) return;

    textarea.style.height = `${textarea.scrollHeight}px`;
    if (avoidSetInput) return;
    setTextareaInput(textarea.value);
    setTextareaInput(textarea.value);
  };

  const onSubmit = (e: Event) => {
    e.preventDefault();
    const textAreaValue = textareaInput();
    if (!textAreaValue) return;

    //set the search query in localStorage;
    updateSearchQueriesInStorage(textAreaValue, searchQueriesFromStorage());

    const searchQuery = encodeURIComponent(
      textAreaValue.length > 3800
        ? textAreaValue.slice(0, 3800)
        : textAreaValue,
    );

    const activeComboBoxFilters = comboBoxSections().flatMap((section) => {
      return {
        name: section.name,
        items: section.comboboxItems.filter((item) => item.selected),
      };
    });

    // Create an array of strings in the format "name=item,item,item"
    const filterStrings = activeComboBoxFilters
      .map((filter) => {
        const itemString = filter.items.map((item) => item.name).join(",");

        if (!itemString) return;

        return `${filter.name}=${itemString}`;
      })
      .filter((item) => item);

    // Join the filter strings with commas
    const filters = filterStrings.join("&");

    window.location.href = props.collectionID
      ? `/collection/${props.collectionID}?q=${searchQuery}` +
        (filters ? `&${filters}` : "") +
        (timeRange().start ? `&start=${timeRange().start}` : "") +
        (timeRange().end ? `&end=${timeRange().end}` : "") +
        (searchTypes()[0].isSelected ? `&searchType=fulltextsearch` : "")
      : `/search?q=${searchQuery}` +
        (filters ? `&${filters}` : "") +
        (timeRange().start ? `&start=${timeRange().start}` : "") +
        (timeRange().end ? `&end=${timeRange().end}` : "") +
        (searchTypes()[0].isSelected ? `&searchType=fulltextsearch` : "");
  };

  onMount(() => {
    const filters = props.filters;
    const linkFilters = filters.link;
    const tagSetFilters = filters.tagSet;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const metadataFilters = filters.metadataFilters;

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
    const savedCustomFilters: any = JSON.parse(
      window.localStorage.getItem("savedCustomFilters") ??
        JSON.stringify({
          tagSet: [],
          link: [],
          metadataFilters: {},
        }),
    );

    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    if (!("tagSet" in Object.keys(savedCustomFilters))) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      savedCustomFilters.tagSet = [];
    }
    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    if (!("link" in Object.keys(savedCustomFilters))) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      savedCustomFilters.link = [];
    }

    setComboBoxSections((prev) => {
      return prev.map((section) => {
        if (section.name === "link") {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
          const savedLinkFilters = savedCustomFilters.link;
          return {
            ...section,
            comboboxItems: [
              ...section.comboboxItems.map((item) => {
                return {
                  ...item,
                  selected: linkFilters.includes(item.name),
                };
              }),
              // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any
              ...savedLinkFilters.map((item: any) => {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return {
                  ...item,
                  custom: true,
                  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument
                  selected: linkFilters.includes(item.name),
                };
              }),
            ],
          };
        } else if (section.name === "Tag Set") {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
          const savedTagSetFilters = savedCustomFilters.tagSet;
          return {
            ...section,
            comboboxItems: [
              ...section.comboboxItems.map((item) => {
                return {
                  ...item,
                  selected: tagSetFilters.includes(item.name),
                };
              }),
              // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any
              ...savedTagSetFilters.map((item: any) => {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return {
                  ...item,
                  custom: true,
                  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-unsafe-member-access
                  selected: tagSetFilters.includes(item.name),
                };
              }),
            ],
          };
        }

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
        const metadataSection = metadataFilters[section.name] ?? [];
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const savedMetadataSection =
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          savedCustomFilters.metadataFilters[section.name] ?? [];

        return {
          ...section,
          comboboxItems: [
            ...section.comboboxItems.map((item) => {
              return {
                ...item,
                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
                selected: metadataSection.includes(item.name),
              };
            }),
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any
            ...savedMetadataSection.map((item: any) => {
              // eslint-disable-next-line @typescript-eslint/no-unsafe-return
              return {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
                ...item,
                custom: true,
                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
                selected: metadataSection.includes(item.name),
              };
            }),
          ],
        };
      });
    });
  });

  createEffect(() => {
    const comboBoxSet = comboBoxSections();

    const tagSetFilters = comboBoxSet
      .find((section) => section.name === "Tag Set")
      ?.comboboxItems.filter((item) => item.custom);

    const linkFilters = comboBoxSet
      .find((section) => section.name === "link")
      ?.comboboxItems.filter((item) => item.custom);

    let metadataFilters = {};
    comboBoxSet.forEach((section) => {
      if (section.name !== "Tag Set" && section.name !== "link") {
        metadataFilters = {
          ...metadataFilters,
          [section.name]: section.comboboxItems.filter((item) => item.custom),
        };
      }
    });

    const filters = {
      tagSet: tagSetFilters ?? [],
      link: linkFilters ?? [],
      metadataFilters: metadataFilters,
    };

    window.localStorage.setItem("savedCustomFilters", JSON.stringify(filters));
  });

  createEffect(() => {
    resizeTextarea(
      document.getElementById(
        "search-query-textarea",
      ) as HTMLTextAreaElement | null,
      false,
    );
    setSearchTypes((prev) => {
      return prev.map((item) => {
        if (props.searchType == item.route) {
          return { ...item, isSelected: true };
        } else {
          return { ...item, isSelected: false };
        }
      });
    });
  });

  createEffect(() => {
    const shouldNotRun = textareaInput() || textareaFocused();

    if (shouldNotRun) {
      return;
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    const textArray: string[] = import.meta.env.PUBLIC_SEARCH_QUERIES.split(
      ",",
    );

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
        timeoutRefOne = setTimeout(typeText, 1000);
      } else if (isDeleting && currentCharIndex === 0) {
        isDeleting = false;
        currentTextIndex = (currentTextIndex + 1) % textArray.length;
        timeoutRefTwo = setTimeout(typeText, typingSpeed);
      } else {
        const speed = isDeleting ? deleteSpeed : typingSpeed;
        timeoutRefThree = setTimeout(typeText, speed);
      }
    };

    typeText();

    onCleanup(() => {
      clearTimeout(timeoutRefOne);
      clearTimeout(timeoutRefTwo);
      clearTimeout(timeoutRefThree);
    });
  });

  // contains the logic for the feeling lucky button
  createEffect(() => {
    let hoverTimeout = 0;

    const feelingRandom = () => {
      setFeelingLuckyText((prev) => {
        const arr = prev.slice();
        for (let i = arr.length - 1; i > 0; i--) {
          const j = Math.floor(Math.random() * (i + 1));
          [arr[i], arr[j]] = [arr[j], arr[i]];
        }
        if (arr[arr.length - 1] === prev[prev.length - 1]) {
          [arr[arr.length - 1], arr[0]] = [arr[0], arr[arr.length - 1]];
        }
        return arr;
      });

      hoverTimeout = setTimeout(() => {
        setFeelingLuckySpinning(false);
        hoverTimeout = 0;
      }, 300);
    };

    const feelingLucky = () => {
      clearTimeout(hoverTimeout);
      hoverTimeout = 0;
      setFeelingLuckySpinning(false);
    };

    const spinning = feelingLuckySpinning();
    if (spinning) {
      if (hoverTimeout) {
        return;
      }
      feelingRandom();
      return;
    }
    feelingLucky();
  });

  const handleHistoryClick = (e: Event, title: string) => {
    setTextareaInput(title);
    onSubmit(e);
    setSearchHistoryList([]);
  };

  return (
    <div class="w-full">
      <form class="w-full space-y-4 dark:text-white" onSubmit={onSubmit}>
        <div class="relative flex">
          <div
            class={`flex w-full justify-center space-x-2 rounded-md bg-neutral-100 px-4 py-1 pr-[10px] dark:bg-neutral-700 ${
              textareaInput().length && searchHistoryList().length
                ? "rounded-bl-none rounded-br-none"
                : ""
            }`}
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
              value={
                textareaInput() ||
                (textareaFocused() ? textareaInput() : typewriterEffect())
              }
              onInput={(e) => {
                resizeTextarea(e.target, false);
                updateSearchHistory(e.target.value, searchQueriesFromStorage());
              }}
              onKeyDown={(e) => {
                if (
                  ((e.ctrlKey || e.metaKey) && e.key === "Enter") ||
                  (!e.shiftKey && e.key === "Enter")
                ) {
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
                  resizeTextarea(
                    document.getElementById(
                      "search-query-textarea",
                    ) as HTMLTextAreaElement,
                    true,
                  );
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
          <Show when={textareaInput().length && searchHistoryList().length}>
            <div class="absolute left-0 top-[99%] z-50 ml-0 w-full rounded-md rounded-tl-none rounded-tr-none bg-neutral-100 pb-1 dark:bg-neutral-700">
              <div class="mx-4 my-1 h-px bg-[#808080]" />
              <For each={searchHistoryList()}>
                {(title) => (
                  <div
                    class="flex w-full cursor-pointer items-center justify-start space-x-2 rounded px-4 py-[6px] pr-[10px] font-bold text-magenta-500 hover:bg-neutral-200 dark:hover:bg-neutral-800"
                    onClick={(e) => handleHistoryClick(e, title)}
                  >
                    <AiOutlineClockCircle class="mr-3 h-5 w-5 fill-black dark:fill-white" />
                    {title}
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
        <div class="flex space-x-2">
          <Show
            when={comboBoxSections().find(
              (comboboxSection) => comboboxSection.name,
            )}
          >
<<<<<<< HEAD
            <button
              classList={{
                "flex items-center space-x-1 text-sm p-1 rounded": true,
                "bg-neutral-200 dark:bg-neutral-700": showFilters(),
              }}
              onClick={(e) => {
                e.preventDefault();
                setShowFilters(!showFilters());
              }}
            >
              <span>Filters</span>
            </button>
=======
            <Popover defaultOpen={false} class="relative">
              {({ isOpen, setState }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle filters"
                    type="button"
                    class="flex items-center space-x-1 text-sm "
                  >
                    <span>Filters</span>{" "}
                    <svg
                      fill="currentColor"
                      stroke-width="0"
                      style={{ overflow: "visible", color: "currentColor" }}
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 "
                      height="1em"
                      width="1em"
                      xmlns="http://www.w3.org/2000/svg"
                    >
                      <path d="M2 5.56L2.413 5h11.194l.393.54L8.373 11h-.827L2 5.56z" />
                    </svg>
                  </PopoverButton>
                  <Show when={isOpen()}>
                    <PopoverPanel
                      unmount={false}
                      class="absolute z-10 mt-2 h-fit w-fit rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                    >
                      <Menu class="h-0">
                        <MenuItem class="h-0" as="button" aria-label="Empty" />
                      </Menu>
                      <div class="flex w-full min-w-full space-x-2">
                        <For each={comboBoxSections()}>
                          {(comboBoxSection) => (
                            <Combobox
                              sectionName={comboBoxSection.name}
                              comboBoxSections={comboBoxSections}
                              setComboboxSections={setComboBoxSections}
                              setPopoverOpen={setState}
                            />
                          )}
                        </For>
                      </div>
                    </PopoverPanel>
                  </Show>
                </>
              )}
            </Popover>
>>>>>>> 0c8dfaf (feature: UI for hybrid weights, bugfix: 0/1 vals for semantic/full-text weights)
          </Show>
          <Popover defaultOpen={false} class="relative">
            {({ isOpen, setState }) => (
              <>
                <PopoverButton
                  aria-label="Toggle filters"
                  type="button"
                  class="flex items-center space-x-1 p-1 text-sm"
                >
                  <span>Search Type</span>{" "}
                  <svg
                    fill="currentColor"
                    stroke-width="0"
                    style={{ overflow: "visible", color: "currentColor" }}
                    viewBox="0 0 16 16"
                    class="h-3.5 w-3.5 "
                    height="1em"
                    width="1em"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path d="M2 5.56L2.413 5h11.194l.393.54L8.373 11h-.827L2 5.56z" />
                  </svg>
                </PopoverButton>
                <Show when={isOpen()}>
                  <PopoverPanel
                    unmount={false}
                    class="absolute z-10 mt-2 h-fit w-[180px]  rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                  >
                    <Menu class="ml-1 space-y-1">
                      <For each={searchTypes()}>
                        {(option) => {
                          if (
                            props.collectionID &&
                            option.route === "hybrid_search"
                          ) {
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
          <Show
            when={
              searchTypes().find((type) => type.isSelected)?.route ===
              "hybrid_search"
            }
          >
            <Popover defaultOpen={false} class="relative">
              {({ isOpen }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle filters"
                    type="button"
                    class="flex items-center space-x-1 text-sm"
                  >
                    <span>Weight</span>{" "}
                    <svg
                      fill="currentColor"
                      stroke-width="0"
                      style={{ overflow: "visible", color: "currentColor" }}
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 "
                      height="1em"
                      width="1em"
                      xmlns="http://www.w3.org/2000/svg"
                    >
                      <path d="M2 5.56L2.413 5h11.194l.393.54L8.373 11h-.827L2 5.56z" />
                    </svg>
                  </PopoverButton>
                  <Show when={isOpen()}>
                    <PopoverPanel
                      unmount={false}
                      class="absolute z-10 mt-2 h-fit w-[180px]  rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                    >
                      <label
                        for="minmax-range"
                        class="mb-2 block space-y-1 text-sm font-medium"
                      >
                        <div>Semantic Weight {semanticWeight()}</div>
                      </label>
                      <input
                        id="minmax-range"
                        type="range"
                        min="0"
                        max="1"
                        value={semanticWeight()}
                        step="0.1"
                        class="h-2 w-full cursor-pointer appearance-none rounded-lg bg-neutral-100 dark:bg-neutral-700"
                        onInput={(e) =>
                          setSemanticWeight(e.currentTarget.value)
                        }
                      />
                    </PopoverPanel>
                  </Show>
                </>
              )}
            </Popover>
          </Show>
        </div>
        <Show when={showFilters()}>
          <div class="flex gap-x-2">
            <For each={comboBoxSections()}>
              {(comboBoxSection) => (
                <Popover defaultOpen={false} class="relative">
                  {({ isOpen, setState }) => (
                    <>
                      <PopoverButton
                        aria-label="Toggle filters"
                        type="button"
                        class="flex items-center space-x-1 text-sm "
                      >
                        <span>{comboBoxSection.name}</span>{" "}
                        <svg
                          fill="currentColor"
                          stroke-width="0"
                          style={{
                            overflow: "visible",
                            color: "currentColor",
                          }}
                          viewBox="0 0 16 16"
                          class="h-3.5 w-3.5 "
                          height="1em"
                          width="1em"
                          xmlns="http://www.w3.org/2000/svg"
                        >
                          <path d="M2 5.56L2.413 5h11.194l.393.54L8.373 11h-.827L2 5.56z" />
                        </svg>
                      </PopoverButton>
                      <Show when={isOpen()}>
                        <PopoverPanel
                          unmount={false}
                          class="absolute z-10 mt-2 h-fit w-fit rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                        >
                          <Menu class="h-0">
                            <MenuItem
                              class="h-0"
                              as="button"
                              aria-label="Empty"
                            />
                          </Menu>
                          <div class="flex w-full min-w-full space-x-2">
                            <Switch>
                              <Match when={comboBoxSection.name != "Date"}>
                                <Combobox
                                  sectionName={comboBoxSection.name}
                                  comboBoxSections={comboBoxSections}
                                  setComboboxSections={setComboBoxSections}
                                  setPopoverOpen={setState}
                                />
                              </Match>
                              <Match when={comboBoxSection.name == "Date"}>
                                <DatePicker
                                  sectionName={comboBoxSection.name}
                                  timeRange={timeRange}
                                  setTimeRange={setTimeRange}
                                  setPopoverOpen={setState}
                                />
                              </Match>
                            </Switch>
                          </div>
                        </PopoverPanel>
                      </Show>
                    </>
                  )}
                </Popover>
              )}
            </For>
          </div>
        </Show>
        <Show when={!props.query && !props.collectionID}>
          <div class="flex justify-center space-x-4 sm:gap-y-0 sm:space-x-2 sm:px-6">
            <button
              class="w-fit rounded bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700"
              type="submit"
            >
              Search
            </button>
            <Show when={createEvidenceFeature}>
              <a
                class="w-fit rounded bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700"
                href="/create"
              >
                Create
              </a>
            </Show>
            <Show when={feelingSuffixes.length > 0}>
              <a
                class="relative hidden h-[40px] overflow-hidden rounded bg-neutral-100 p-2 text-center transition-width duration-1000 hover:bg-neutral-100 dark:bg-neutral-700 sm:block"
                href={`/search?q=I'm feeling ${
                  feelingLuckyText().findLast(() => true) ?? ""
                }`}
                onMouseEnter={() => {
                  setFeelingLuckySpinning(true);
                }}
                onMouseLeave={() => {
                  setFeelingLuckySpinning(false);
                }}
              >
                <Show when={feelingLuckySpinning()}>
                  <span class="block h-[40px] animate-scrollup overflow-hidden">
                    <For each={feelingLuckyText()}>
                      {(text) => <p>I'm Feeling {text}</p>}
                    </For>
                  </span>
                </Show>
                <Show when={!feelingLuckySpinning()}>
                  <span>
                    I'm Feeling {feelingLuckyText().findLast(() => true)}
                  </span>
                </Show>
              </a>
            </Show>
          </div>
        </Show>
      </form>
    </div>
  );
};

export default SearchForm;
