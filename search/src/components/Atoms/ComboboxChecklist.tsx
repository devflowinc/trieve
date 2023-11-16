import { Accessor, For, Setter, createMemo, createSignal } from "solid-js";
import { FaSolidCheck } from "solid-icons/fa";
import { BiRegularSearchAlt } from "solid-icons/bi";

export interface ComboboxItem {
  name: string;
  custom?: boolean;
  selected?: boolean;
}
export interface ComboboxSection {
  name: string;
  comboboxItems: ComboboxItem[];
}

export interface ComboboxProps {
  sectionName: string;
  comboBoxSections: Accessor<ComboboxSection[]>;
  setComboboxSections: Setter<ComboboxSection[]>;
  setPopoverOpen: (newState: boolean) => void;
}

export const Combobox = (props: ComboboxProps) => {
  const [usingPanel, setUsingPanel] = createSignal(false);
  const [inputValue, setInputValue] = createSignal("");

  const filteredItems = createMemo(() => {
    const currentSection = props
      .comboBoxSections()
      .find((section) => section.name === props.sectionName);
    if (!currentSection) return [];

    const matchingItems = currentSection.comboboxItems.filter((option) => {
      return option.name.toLowerCase().includes(inputValue().toLowerCase());
    });

    return [
      ...matchingItems,
      {
        name: "+ Add custom filter",
        selected: false,
      },
    ];
  });

  const handleClick = (name: string, selected: boolean | undefined) => {
    const curentSection = props
      .comboBoxSections()
      .find((section) => section.name === props.sectionName);
    if (!curentSection) return;

    if (name === "+ Add custom filter") {
      const newName = inputValue();
      props.setComboboxSections((prev) =>
        prev.map((section) => {
          if (section.name !== curentSection.name) {
            return section;
          }

          return {
            name: section.name,
            comboboxItems: [
              ...section.comboboxItems,
              { name: newName, custom: true, selected: true },
            ],
          };
        }),
      );
    }

    props.setComboboxSections((prev) =>
      prev.map((section) => {
        if (section.name !== curentSection.name) {
          return section;
        }

        return {
          name: section.name,
          comboboxItems: section.comboboxItems.map((comboboxItem) => {
            if (comboboxItem.name === name) {
              return {
                name: comboboxItem.name,
                custom: comboboxItem.custom,
                selected: !selected,
              };
            }
            return comboboxItem;
          }),
        };
      }),
    );

    props.setPopoverOpen(true);
  };

  return (
    <div class="w-full min-w-[165px]">
      <div class="mb-1 text-center text-sm font-semibold">
        {props.sectionName}
      </div>
      <div class="flex w-fit items-center space-x-2 rounded bg-white px-2 focus:outline-black dark:bg-neutral-600 dark:focus:outline-white">
        <BiRegularSearchAlt class="h-5 w-5 fill-current text-neutral-500 dark:text-neutral-400" />
        <input
          class="w-full bg-transparent focus:outline-none"
          type="text"
          onBlur={() => !usingPanel()}
          value={inputValue()}
          onInput={(e) => setInputValue(e.currentTarget.value)}
          placeholder="Search"
        />
      </div>
      <div
        class="mt-1 max-h-[40vh] w-full transform space-y-1 overflow-y-auto rounded px-2 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
        onMouseEnter={() => {
          setUsingPanel(true);
        }}
        onMouseLeave={() => {
          setUsingPanel(false);
        }}
      >
        <For each={filteredItems()}>
          {({ name, selected }) => {
            return (
              <button
                type="button"
                classList={{
                  "flex w-full items-center justify-between rounded p-1 bg-neutral-100/20 dark:bg-neutral-700/20 dark:bg-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700":
                    true,
                  "bg-neutral-300 dark:bg-neutral-900": selected,
                }}
                onClick={() => handleClick(name, selected)}
              >
                <div class="w-full break-all text-left">{name}</div>
                {selected && (
                  <span>
                    <FaSolidCheck class="fill-current text-xl" />
                  </span>
                )}
              </button>
            );
          }}
        </For>
      </div>
    </div>
  );
};
