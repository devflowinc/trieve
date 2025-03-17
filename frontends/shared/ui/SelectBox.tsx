import { FaRegularCircleQuestion, FaSolidCheck } from "solid-icons/fa";
import { TbSelector } from "solid-icons/tb";
import { createEffect, createSignal, For, JSX, Show } from "solid-js";
import {
  DisclosureStateChild,
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "terracotta";
import createFuzzySearch from "@nozbe/microfuzz";
import { cn } from "../utils";
import { Tooltip } from "./Tooltip";

interface SelectProps<T> {
  options: T[];
  display: (option: T) => string;
  displayElement?: (option: T) => JSX.Element;
  selected: T;
  onSelected: (option: T) => void;
  class?: string;
  label?: JSX.Element;
  id?: string;
  tooltipText?: string;
  tooltipDirection?: "top" | "bottom" | "left" | "right";
}

export const Select = <T,>(props: SelectProps<T>) => {
  const [open, setOpen] = createSignal(false);
  const [searchTerm, setSearchTerm] = createSignal("");
  const [searchResults, setSearchResults] = createSignal<T[]>([]);

  createEffect(() => {
    if (searchTerm() === "") {
      setSearchResults(props.options);
    } else {
      const fuzzy = createFuzzySearch(props.options, {
        getText: (item: T) => {
          return [props.display(item)];
        },
      });
      const results = fuzzy(searchTerm());
      console.log("RESULTS", results);
      setSearchResults(results.map((result) => result.item));

      const input = document.getElementById(`${props.id}-search`);
      if (input) {
        setTimeout(() => {
          input.focus();
        }, 500);
        setTimeout(() => {
          input.focus();
        }, 1000);
      }
    }
  });

  return (
    <>
      <div class="flex items-center gap-2">
        <Show when={props.label}>{(label) => label()}</Show>
        <Show when={props.tooltipText}>
          {(tooltipText) => (
            <Tooltip
              tooltipText={tooltipText()}
              body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
            />
          )}
        </Show>
      </div>
      <Listbox
        class={cn(
          `bg-neutral-200/70 min-w-[100px] relative border rounded border-neutral-300`,
          props.class,
        )}
        value={props.selected}
        defaultOpen={false}
        onClose={() => setSearchTerm("")}
      >
        <ListboxButton
          type="button"
          class="flex py-1 text-sm px-3 w-full justify-between gap-2 items-center"
          onClick={() => setOpen(!open())}
        >
          {props.displayElement
            ? props.displayElement(props.selected)
            : props.display(props.selected)}
          <TbSelector />
        </ListboxButton>
        <DisclosureStateChild>
          {({ isOpen }): JSX.Element => (
            <Show when={isOpen()}>
              <div class="relative w-full">
                <ListboxOptions
                  unmount={false}
                  tabIndex={0}
                  class="absolute z-40 shadow mt-1 max-h-[70vh] w-fit max-w-[300px] overflow-y-auto overflow-x-none rounded-md bg-white text-base outline outline-1 outline-gray-300 ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm"
                >
                  <Show when={props.options.length > 5}>
                    <input
                      id={`${props.id}-search`}
                      placeholder="Search..."
                      class="mb-2 flex mx-auto items-center rounded bg-neutral-100 p-1 mt-2 text-sm text-black outline-none dark:bg-neutral-700 dark:hover:text-white dark:focus:text-white"
                      onInput={(e) => {
                        setSearchTerm(e.target.value);
                      }}
                      value={searchTerm()}
                    />
                  </Show>
                  <For each={searchResults().slice(0, 500)}>
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
                              "group-hover:bg-magenta-50 group-hover:cursor-pointer whitespace-nowrap flex p-2 justify-between items-center gap-2 group-hover:text-magenta-900 relative cursor-default select-none ":
                                true,
                            }}
                            onClick={() => {
                              props.onSelected(option);
                              setOpen(false);
                            }}
                          >
                            <span>
                              {props.displayElement
                                ? props.displayElement(option)
                                : props.display(option)}
                            </span>
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
          )}
        </DisclosureStateChild>
      </Listbox>
    </>
  );
};
