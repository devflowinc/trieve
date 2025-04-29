/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { AiOutlineCheck } from "solid-icons/ai";
import { TbSelector } from "solid-icons/tb";
import type { JSX } from "solid-js";
import { For, createSignal, Show, createEffect } from "solid-js";
import {
  DisclosureStateChild,
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "terracotta";

export interface Item {
  id: string;
  name: string;
}

export function MultiSelect<I extends Item>(props: {
  disabled?: boolean;
  items: I[];
  selected?: I[];
  setSelected: (selected: I[]) => void;
}): JSX.Element {
  const [selected, setSelectedItems] = createSignal<I[]>([]);

  createEffect(() => {
    if (props.selected) {
      setSelectedItems(props.selected);
    }
  });

  const setSelected = (selected: I[]) => {
    props.setSelected(selected);
    setSelectedItems(selected);
  };

  // If a selected item is not in the pool of items,
  // reset the list of selected items, this likely
  // needs to be fixed in the terracotta library
  createEffect(() => {
    const selectedIds = selected().map((item) => item.id);
    const newSelected = props.items.filter((item) =>
      selectedIds.includes(item.id),
    );
    if (newSelected.length !== selected().length) {
      setSelected(newSelected);
    }
  });

  return (
    <Listbox
      disabled={props.disabled}
      multiple
      toggleable
      defaultOpen={false}
      value={selected()}
      onSelectChange={setSelected}
      onClick={(e: MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();
      }}
      as="div"
    >
      <div class="relative">
        <ListboxButton class="relative min-w-[300px] max-w-[400px] cursor-default rounded-lg bg-white py-2 pl-3 pr-10 text-left outline outline-1 outline-gray-300 focus-visible:border-indigo-500 focus-visible:ring-2 focus-visible:ring-white focus-visible:ring-opacity-75 focus-visible:ring-offset-2 focus-visible:ring-offset-magenta-300 disabled:opacity-50 sm:text-sm">
          <div class="flex flex-wrap gap-1">
            <For
              each={selected()}
              fallback={<span class="block truncate">None selected.</span>}
            >
              {(item): JSX.Element => (
                <span class="inline-flex items-center rounded bg-magenta-100 px-2 py-0.5 text-xs font-medium text-magenta-800 hover:bg-magenta-300">
                  {item.name}
                </span>
              )}
            </For>
          </div>
          <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
            <TbSelector class="h-5 w-5" />
          </span>
        </ListboxButton>
        <DisclosureStateChild>
          {({ isOpen }): JSX.Element => (
            <Show when={isOpen()}>
              <ListboxOptions
                unmount={false}
                class="absolute z-[99999] mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base outline outline-1 outline-gray-300 ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm"
              >
                <For each={props.items}>
                  {(item): JSX.Element => (
                    <ListboxOption
                      class="group rounded-md p-1 focus:outline-none"
                      value={item}
                    >
                      {({ isSelected }): JSX.Element => {
                        return (
                          <div
                            classList={{
                              "bg-magenta-100 text-magenta-900": isSelected(),
                              "text-gray-900": !isSelected(),
                              "group-hover:bg-magenta-50 group-hover:text-magenta-900 relative cursor-default select-none py-2 pl-10 pr-4 rounded-md":
                                true,
                            }}
                          >
                            <span
                              classList={{
                                "font-medium": isSelected(),
                                "font-normal": !isSelected(),
                                "block truncate": true,
                              }}
                            >
                              {item.name}
                            </span>
                            <Show when={isSelected()}>
                              <span class="absolute inset-y-0 left-0 flex items-center pl-3">
                                <AiOutlineCheck class="h-5 w-5" />
                              </span>
                            </Show>
                          </div>
                        );
                      }}
                    </ListboxOption>
                  )}
                </For>
              </ListboxOptions>
            </Show>
          )}
        </DisclosureStateChild>
      </div>
    </Listbox>
  );
}
