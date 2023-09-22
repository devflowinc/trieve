import "./SearchForm.css";
import { BiRegularSearch, BiRegularX } from "solid-icons/bi";
import { For, Show, createEffect, createSignal, onMount } from "solid-js";
import {
  Combobox,
  ComboboxItem,
  ComboboxSection,
} from "../Atoms/ComboboxChecklist";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
  Transition,
} from "solid-headless";
import { FaSolidCheck } from "solid-icons/fa";
import type { Filters } from "../ResultsPage";

const parseEnvComboboxItems = (data: string | undefined): ComboboxItem[] => {
  const names = data?.split(",");
  if (!names) return [];
  return names.map((name) => {
    return {
      name: name,
    };
  });
};

const SearchForm = (props: {
  query?: string;
  filters: Filters;
  searchType: string;
  collectionID?: string;
}) => {
  const tag_set_items = parseEnvComboboxItems(
    import.meta.env.PUBLIC_TAG_SET_ITEMS,
  );
  const links_items = parseEnvComboboxItems(import.meta.env.PUBLIC_LINK_ITEMS);
  const create_evidence_feature =
    import.meta.env.PUBLIC_CREATE_EVIDENCE_FEATURE !== "off";
  const allTexts = (
    import.meta.env.PUBLIC_LUCKY_ITEMS || "Lorem,Ipsum,Lorem,Ipsum,Lorem,Ipsum,LoremIpsumLoremIpsumLoremIpsum"
  )
    .split(",")
    .filter((i: string | null | undefined) => i)
    .map((i: string) => i.trim());

  const filterDataTypeComboboxSections: ComboboxSection[] = [
    {
      name: "Tag Set",
      comboboxItems: tag_set_items,
    },
  ];
  const filterLinkComboboxSections: ComboboxSection[] = [
    {
      name: "Links",
      comboboxItems: links_items,
    },
  ];

  const [searchTypes, setSearchTypes] = createSignal([
    { name: "Full Text", isSelected: false, route: "fulltextsearch" },
    { name: "Semantic", isSelected: true, route: "search" },
  ]);
  // eslint-disable-next-line solid/reactivity
  const [textareaInput, setTextareaInput] = createSignal(props.query ?? "");

  const [filterDataTypes, setFilterDataTypes] = createSignal<ComboboxSection[]>(
    filterDataTypeComboboxSections,
  );

  const [filterLinks, setFilterLinks] = createSignal<ComboboxSection[]>(
    filterLinkComboboxSections,
  );
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const customDataTypeFilters = JSON.parse(
    localStorage.getItem("customDatasetFilters") ?? "[]",
  );
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const customLinkFilters = JSON.parse(
    localStorage.getItem("customLinksFilters") ?? "[]",
  );
  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
  if (Object.keys(customDataTypeFilters).length > 0) {
    setFilterDataTypes((prev) => {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      customDataTypeFilters.custom = true;
      const newComboboxItems = [
        ...prev[0].comboboxItems,
        customDataTypeFilters,
      ];
      return [
        {
          name: prev[0].name,
          comboboxItems: newComboboxItems,
        },
      ];
    });
  }
  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
  if (Object.keys(customLinkFilters).length > 0) {
    setFilterLinks((prev) => {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      customLinkFilters.custom = true;
      const newComboboxItems = [...prev[0].comboboxItems, customLinkFilters];
      return [
        {
          name: prev[0].name,
          comboboxItems: newComboboxItems,
        },
      ];
    });
  }

  // eslint-disable-next-line solid/reactivity
  const initialDataTypeFilters = filterDataTypes().flatMap((section) =>
    section.comboboxItems.filter((item) =>
      // eslint-disable-next-line solid/reactivity
      props.filters.dataTypes.includes(item.name),
    ),
  );
  // eslint-disable-next-line solid/reactivity
  const initialLinkFilters = filterLinks().flatMap((section) =>
    section.comboboxItems.filter((item) =>
      // eslint-disable-next-line solid/reactivity
      props.filters.links.includes(item.name),
    ),
  );
  const [selectedDataTypeComboboxItems, setDataTypeSelectedComboboxItems] =
    createSignal<ComboboxItem[]>(initialDataTypeFilters);
  const [selectedLinkComboboxItems, setLinkSelectedComboboxItems] =
    createSignal<ComboboxItem[]>(initialLinkFilters);
  const resizeTextarea = (textarea: HTMLTextAreaElement | null) => {
    if (!textarea) return;

    textarea.style.height = `${textarea.scrollHeight}px`;
    setTextareaInput(textarea.value);
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
    const dataTypeFilters = encodeURIComponent(
      selectedDataTypeComboboxItems()
        .map((item) => item.name)
        .join(","),
    );
    const linkFilters = encodeURIComponent(
      selectedLinkComboboxItems()
        .map((item) => item.name)
        .join(","),
    );

    window.location.href = props.collectionID
      ? `/collection/${props.collectionID}?q=${searchQuery}` +
        (dataTypeFilters ? `&datatypes=${dataTypeFilters}` : "") +
        (linkFilters ? `&links=${linkFilters}` : "") +
        (searchTypes()[0].isSelected ? `&searchType=fulltextsearch` : "")
      : `/search?q=${searchQuery}` +
        (dataTypeFilters ? `&datatypes=${dataTypeFilters}` : "") +
        (linkFilters ? `&links=${linkFilters}` : "") +
        (searchTypes()[0].isSelected ? `&searchType=fulltextsearch` : "");
  };

  createEffect(() => {
    resizeTextarea(
      document.getElementById(
        "search-query-textarea",
      ) as HTMLTextAreaElement | null,
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

  onMount(() => {
    const getLuckyText: () => HTMLAnchorElement | null = function () {
      const text = document.getElementById(
        "lucky-text",
      ) as HTMLAnchorElement | null;
      return text;
    };

    // Scroll in new text and URL from randomized list
    const newText = function (inputText: string) {
      // Timeout is set to the same length as the slideout animation duration
      setTimeout(() => {
        const text = getLuckyText();
        if (text) {
          text.classList.add("text-in");
          text.classList.remove("text-out");
          const button = document.getElementById("lucky-button");
          if (button) {
            if (inputText.length > "Lucky".length) {
              button.classList.add("overflow-x-hidden");
              button.classList.add("widening-animation");
            } else {
              button.classList.remove("overflow-x-hidden");
              button.classList.remove("widening-animation");
            }
          }
          text.textContent = `I'm Feeling ${inputText}`;
          text.href = `/search?q=${inputText}`;
        }
      }, 100);
    };

    // Scroll out current / previous text before scrolling in new text
    const oldText = function () {
      const text = getLuckyText();
      if (text) {
        text.classList.remove("text-in");
        text.classList.add("text-out");
      }
    };

    // Interval object to track where we are in the text rotation
    const updateTextInterval = (() => {
      let interval = 0;
      const getInterval = () => interval;
      const increment = () => interval++;
      const reset = () => (interval = 0);
      return { getInterval, increment, reset };
    })();

    // Customized list of button text & URLs
    const buttonText = (() => {
      const text = allTexts;
      console.log(text);
      // Randomizes buttonText array
      let randomizeText = () => {
        // Fishers-Yates shuffle algorithm
        for (let i = text.length - 1; i > 0; i--) {
          let rando = Math.floor(Math.random() * (i + 1));
          [text[i], text[rando]] = [text[rando], text[i]];
        }
      };
      const getText = () => text;
      return { getText, randomizeText };
    })();

    // Random number generator used to determine how many links we show on rotation
    // as well as the direction the text scrolls
    const randomNumCount = (() => {
      let randomNum: number;

      const setRandomNum = () => {
        const maxNum = buttonText.getText().length - 1;
        const minNum = 2;
        randomNum = Math.floor(Math.random() * (maxNum - minNum + 1)) + minNum;
      };

      const getRandomNum = () => randomNum;

      return { setRandomNum, getRandomNum };
    })();

    // Uses random number generator randomNumCount to determine text scroll direction
    const setTextDirection = function () {
      const randomNum = randomNumCount.getRandomNum();
      const text = getLuckyText();

      if (randomNum % 2 === 0) {
        if (text) text.style.animationDirection = "normal";
      } else {
        if (text) text.style.animationDirection = "reverse";
      }
    };

    // Global variable set in changeText and cleared in resetButton
    let hoverTimeout: any;

    // On mouseleave, reset the button to its original text & URL
    // also reset textInterval in preparation for next hover
    const resetButton = function () {
      updateTextInterval.reset();
      // randomNum = randomNumCount.setRandomNum();
      newText("Lucky");
    };

    // Scrolls from old text to new text for length of randomized interval
    const changeText = function () {
      oldText();
      newText(buttonText.getText()[updateTextInterval.getInterval()]);
      updateTextInterval.increment();

      // hoverTimeout is set to slideout + slidein duration total
      // This allows enough time for text to change between updates
      // Timeout is a named global variable so we can clear it in resetButton upon mouseleave
      if (updateTextInterval.getInterval() < randomNumCount.getRandomNum()) {
        hoverTimeout = setTimeout(changeText, 200);
      }
    };

    // On lucky button hover, set our randomized elements and then
    // call changeText() to scroll through these elements
    const feelingRandom = function () {
      randomNumCount.setRandomNum();
      setTextDirection();
      buttonText.randomizeText();
      changeText();
    };

    // On leaving lucky button, stop scrolling text
    // and reset to original text/URL
    const feelingLucky = function () {
      clearTimeout(hoverTimeout);
      resetButton();
    };

    const button = document.getElementById("lucky-button");
    if (button) {
      console.log("Enetered");
      button.addEventListener("mouseenter", feelingRandom);
      button.addEventListener("mouseleave", feelingLucky);
    }
  });

  return (
    <div class="w-full">
      <form class="w-full space-y-4 dark:text-white" onSubmit={onSubmit}>
        <div class="flex space-x-2">
          <div class="flex w-full justify-center space-x-2 rounded-md bg-neutral-100 px-4 py-1 pr-[10px] dark:bg-neutral-700 ">
            <Show when={!props.query}>
              <BiRegularSearch class="mt-1 h-6 w-6 fill-current" />
            </Show>
            <textarea
              id="search-query-textarea"
              class="scrollbar-track-rounded-md scrollbar-thumb-rounded-md mr-2 h-fit max-h-[240px] w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
              placeholder="Search for cards..."
              value={textareaInput()}
              onInput={(e) => resizeTextarea(e.target)}
              onKeyDown={(e) => {
                if (
                  ((e.ctrlKey || e.metaKey) && e.key === "Enter") ||
                  (!e.shiftKey && e.key === "Enter")
                ) {
                  onSubmit(e);
                }
              }}
              rows="1"
            >
              {textareaInput()}
            </textarea>
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
        </div>
        <div class="flex space-x-2">
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
                <Transition
                  show={isOpen()}
                  enter="transition duration-200"
                  enterFrom="opacity-0"
                  enterTo="opacity-100"
                  leave="transition duration-150"
                  leaveFrom="opacity-100"
                  leaveTo="opacity-0"
                >
                  <PopoverPanel
                    unmount={false}
                    class="absolute z-10 mt-2 h-fit w-fit rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                  >
                    <Menu class="h-0">
                      <MenuItem class="h-0" as="button" aria-label="Empty" />
                    </Menu>
                    <div class="flex w-full min-w-full space-x-2">
                      <Show when={!props.collectionID}>
                        <Combobox
                          selectedComboboxItems={selectedDataTypeComboboxItems}
                          setSelectedComboboxItems={
                            setDataTypeSelectedComboboxItems
                          }
                          comboboxSections={filterDataTypes}
                          setComboboxSections={setFilterDataTypes}
                          setPopoverOpen={setState}
                        />
                      </Show>
                      <Combobox
                        selectedComboboxItems={selectedLinkComboboxItems}
                        setSelectedComboboxItems={setLinkSelectedComboboxItems}
                        comboboxSections={filterLinks}
                        setComboboxSections={setFilterLinks}
                        setPopoverOpen={setState}
                      />
                    </div>
                  </PopoverPanel>
                </Transition>
              </>
            )}
          </Popover>
          <Popover defaultOpen={false} class="relative">
            {({ isOpen, setState }) => (
              <>
                <PopoverButton
                  aria-label="Toggle filters"
                  type="button"
                  class="flex items-center space-x-1 text-sm"
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
                <Transition
                  show={isOpen()}
                  enter="transition duration-200"
                  enterFrom="opacity-0"
                  enterTo="opacity-100"
                  leave="transition duration-150"
                  leaveFrom="opacity-100"
                  leaveTo="opacity-0"
                >
                  <PopoverPanel
                    unmount={false}
                    class="absolute z-10 mt-2 h-fit w-[180px]  rounded-md bg-neutral-200 p-1 shadow-lg dark:bg-neutral-800"
                  >
                    <Menu class="ml-1 space-y-1">
                      <For each={searchTypes()}>
                        {(option) => {
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
                </Transition>
              </>
            )}
          </Popover>
        </div>
        <Show when={!props.query && !props.collectionID}>
          <div class="flex flex-col gap-y-2 sm:flex-row sm:justify-center sm:gap-y-0 sm:space-x-2 sm:px-6">
            <button
              class="w-fit rounded bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
              type="submit"
            >
              Search for Evidence
            </button>
            <Show when={create_evidence_feature}>
              <a
                class="w-fit rounded bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
                href="/create"
              >
                Create Evidence Card
              </a>
            </Show>
            <div
              id="lucky-button"
              class="h-[40px] w-fit overflow-y-hidden rounded bg-neutral-100 p-2 text-center hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            >
              <a href="/search?q=" id="lucky-text">
                I'm Feeling Lucky
              </a>
            </div>
          </div>
        </Show>
      </form>
    </div>
  );
};

export default SearchForm;
