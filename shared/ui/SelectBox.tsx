import { FaSolidCheck } from "solid-icons/fa";
import { TbSelector } from "solid-icons/tb";
import { createSignal, For, JSX, Show } from "solid-js";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "terracotta";

interface SelectProps<T> {
  options: T[];
  display: (option: T) => string;
  selected: T;
  onSelected: (option: T) => void;
  class?: string;
  label?: JSX.Element;
}

export const Select = <T,>(props: SelectProps<T>) => {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <Show when={props.label}>{(label) => label()}</Show>
      <Listbox
        class={`bg-neutral-200/70 min-w-[100px] relative border rounded border-neutral-300 ${props.class}`}
        value={props.selected}
        defaultOpen={false}
      >
        <ListboxButton
          class="flex py-1 text-sm px-3 w-full justify-between gap-2 items-center"
          onClick={() => setOpen(!open())}
        >
          {props.display(props.selected)}
          <TbSelector />
        </ListboxButton>
        <Show when={open()}>
          <div class="relative w-full">
            <ListboxOptions
              unmount={false}
              class="absolute z-40 shadow mt-1 max-h-60 w-full overflow-auto rounded-md bg-white text-base outline outline-1 outline-gray-300 ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm"
            >
              <For each={props.options}>
                {(option): JSX.Element => (
                  <ListboxOption
                    class="group min-w-full w-[max-content] rounded-md focus:outline-none"
                    value={option}
                  >
                    {({ isSelected }): JSX.Element => (
                      <div
                        classList={{
                          "bg-magenta-100 text-magenta-900": isSelected(),
                          "text-gray-900": !isSelected(),
                          "group-hover:bg-magenta-50 whitespace-nowrap flex p-2 justify-between px-2 items-center gap-2 group-hover:text-magenta-900 relative cursor-default select-none ":
                            true,
                        }}
                        onClick={() => {
                          props.onSelected(option);
                          setOpen(false);
                        }}
                      >
                        <span>{props.display(option)}</span>
                        {isSelected() ? (
                          <span
                            classList={{
                              "": true,
                            }}
                          >
                            <FaSolidCheck />
                          </span>
                        ) : null}
                      </div>
                    )}
                  </ListboxOption>
                )}
              </For>
            </ListboxOptions>
          </div>
        </Show>
      </Listbox>
    </>
  );
};
