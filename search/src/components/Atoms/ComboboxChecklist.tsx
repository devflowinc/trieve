import { Accessor, For, Setter, createMemo, createSignal } from "solid-js";
import { FaSolidCheck } from "solid-icons/fa";
import { BiRegularSearchAlt } from "solid-icons/bi";
import { VsClose } from "solid-icons/vs";
export interface ComboboxItem {
  name: string;
  custom?: boolean;
}
export interface ComboboxSection {
  name: string;
  comboboxItems: ComboboxItem[];
}

export interface ComboboxProps {
  comboboxSections: Accessor<ComboboxSection[]>;
  setComboboxSections: Setter<ComboboxSection[]>;
  selectedComboboxItems: Accessor<ComboboxItem[]>;
  setSelectedComboboxItems: Setter<ComboboxItem[]>;
  setPopoverOpen: (newState: boolean) => void;
}

export const Combobox = (props: ComboboxProps) => {
  const [usingPanel, setUsingPanel] = createSignal(false);
  const [inputValue, setInputValue] = createSignal("");
  const [hoverOverCustom, setHoverOverCustom] = createSignal(false);

  const filteredSectionsWithIsSelected = createMemo(() => {
    const selected = props.selectedComboboxItems();
    const sectionsWithSelected = props.comboboxSections().map((section) => {
      return {
        sectionName: section.name,
        sectionSelectedItems: section.comboboxItems.map((option) => {
          const isSelected = selected.some(
            (selectedOption) => selectedOption.name === option.name,
          );
          return {
            ...option,
            isSelected,
          };
        }),
      };
    });

    if (!inputValue()) return sectionsWithSelected;
    return sectionsWithSelected.map((section) => {
      let sectionSelectedItems = section.sectionSelectedItems.filter(
        (option) => {
          return option.name.toLowerCase().includes(inputValue().toLowerCase());
        },
      );

      sectionSelectedItems.push({
        name: "+ Add custom filter",
        isSelected: false,
      });

      return {
        ...section,
        sectionSelectedItems,
      };
    });
  });

  const onSelect = (option: ComboboxItem) => {
    if (option.name === "+ Add custom filter") {
      const newName = inputValue();
      props.setComboboxSections((prev) => {
        const newComboboxItems = [
          ...prev[0].comboboxItems,
          { name: newName, custom: true },
        ];
        return [
          {
            name: prev[0].name,
            comboboxItems: newComboboxItems,
          },
        ];
      });
      props.setSelectedComboboxItems((prev) => {
        return [...prev, { name: newName }];
      });
      localStorage.setItem(
        `custom${props.comboboxSections()[0].name.replace(" ", "")}Filters`,
        JSON.stringify({ name: inputValue() }),
      );
      props.setPopoverOpen(true);
      return;
    }
    props.setSelectedComboboxItems((prev) => {
      const prevIncludesOption = prev.find((prevOption) => {
        return prevOption.name === option.name;
      });
      if (!prevIncludesOption) {
        return [...prev, option];
      }
      return prev.filter((prevOption) => prevOption.name !== option.name);
    });
    props.setPopoverOpen(true);
  };

  const placeholder = createMemo(() => {
    let placeholder = "";
    const selected = props.selectedComboboxItems();
    selected.forEach((option) => {
      placeholder != "" && (placeholder += ", ");
      placeholder += option.name;
    });
    return placeholder;
  });

  const removeCustomItem = (e: Event, customItem: ComboboxItem) => {
    e.preventDefault();
    e.stopPropagation();
    props.setComboboxSections((prev) => {
      const newComboboxItems = prev[0].comboboxItems.filter((comboboxItem) => {
        return comboboxItem.name !== customItem.name;
      });
      return [
        {
          name: prev[0].name,
          comboboxItems: newComboboxItems,
        },
      ];
    });
    props.setSelectedComboboxItems((prev) => {
      return prev.filter((prevOption) => prevOption.name !== customItem.name);
    });
    localStorage.removeItem(
      `custom${props.comboboxSections()[0].name.replace(" ", "")}Filters`,
    );
    props.setPopoverOpen(true);
  };

  return (
    <div class="w-full min-w-[150px]">
      <div class="mb-1 text-center text-sm font-semibold">
        {filteredSectionsWithIsSelected()[0].sectionName}{" "}
      </div>
      <div class="flex w-fit items-center space-x-2 rounded bg-white px-2 focus:outline-black dark:bg-neutral-600 dark:focus:outline-white">
        <BiRegularSearchAlt class="h-5 w-5 fill-current text-neutral-500 dark:text-neutral-400" />
        <input
          class="w-full bg-transparent focus:outline-none"
          type="text"
          onBlur={() => !usingPanel()}
          value={inputValue()}
          onInput={(e) => setInputValue(e.currentTarget.value)}
          placeholder={placeholder()}
        />
      </div>
      <div
        class="scrollbar-track-rounded-md scrollbar-thumb-rounded-md mt-1 max-h-[40vh] w-full transform overflow-y-auto rounded px-2 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
        onMouseEnter={() => {
          setUsingPanel(true);
        }}
        onMouseLeave={() => {
          setUsingPanel(false);
        }}
      >
        <For each={filteredSectionsWithIsSelected()}>
          {({ sectionSelectedItems }) => {
            return (
              <div>
                <div class="ml-1 space-y-1">
                  <For each={sectionSelectedItems}>
                    {(option) => {
                      const onClick = (e: Event) => {
                        e.preventDefault();
                        e.stopPropagation();
                        onSelect(option);
                      };

                      return (
                        <button
                          type="button"
                          classList={{
                            "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white":
                              true,
                            "bg-neutral-300 dark:bg-neutral-900":
                              option.isSelected,
                          }}
                          onClick={onClick}
                          onMouseEnter={() => {
                            option.custom && setHoverOverCustom(true);
                          }}
                          onMouseLeave={() => {
                            option.custom && setHoverOverCustom(false);
                          }}
                        >
                          <div class="flex flex-row justify-start space-x-2">
                            <span class="text-left">{option.name}</span>
                          </div>
                          {option.isSelected && (
                            <span>
                              <FaSolidCheck class="fill-current text-xl" />
                            </span>
                          )}
                          {hoverOverCustom() &&
                            !option.isSelected &&
                            option.custom && (
                              <span
                                onClick={(e) => removeCustomItem(e, option)}
                              >
                                <VsClose class="fill-current text-xl" />
                              </span>
                            )}
                        </button>
                      );
                    }}
                  </For>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};
