import { Show, JSX, createEffect, createSignal, on } from "solid-js";
import { RangeFilter } from "../types";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { TbSelector } from "solid-icons/tb";
import { cn } from "../utils";
import { Tooltip } from "./Tooltip";

export interface RangePickerProps {
  onChange: (filter: RangeFilter) => void;
  label?: JSX.Element;
  tooltipText?: string;
  class?: string;
  id?: string;
}

export const RangePicker = (props: RangePickerProps) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [startNumber, setStartNumber] = createSignal<number | undefined>(
    undefined
  );
  const [endNumber, setEndNumber] = createSignal<number | undefined>(undefined);

  const displayText = () => {
    if (startNumber() === undefined && endNumber() === undefined) {
      return "Select range...";
    } else if (startNumber() !== undefined && endNumber() === undefined) {
      return `≥ ${startNumber()}`;
    } else if (startNumber() === undefined && endNumber() !== undefined) {
      return `< ${endNumber()}`;
    } else {
      return `${startNumber()} - ${endNumber()}`;
    }
  };

  createEffect(
    on([startNumber, endNumber], () => {
      props.onChange({ gte: startNumber(), lt: endNumber() });
    })
  );

  const handleClickOutside = (e: MouseEvent) => {
    const popover = document.getElementById(`${props.id || "range"}-popover`);
    const button = document.getElementById(`${props.id || "range"}-button`);

    if (
      popover &&
      button &&
      !popover.contains(e.target as Node) &&
      !button.contains(e.target as Node)
    ) {
      setIsOpen(false);
    }
  };

  createEffect(() => {
    if (isOpen()) {
      document.addEventListener("mousedown", handleClickOutside);
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  });

  return (
    <div class="flex flex-col">
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

      <div class="relative">
        <button
          id={`${props.id || "range"}-button`}
          type="button"
          class={cn(
            `bg-neutral-200/70 min-w-[100px] border rounded border-neutral-300 flex py-1 text-sm px-3 w-full justify-between gap-2 items-center`,
            props.class
          )}
          onClick={() => setIsOpen(!isOpen())}
        >
          <span>{displayText()}</span>
          <TbSelector />
        </button>

        <Show when={isOpen()}>
          <div
            id={`${props.id || "range"}-popover`}
            class="absolute z-40 shadow mt-1 p-4 w-full bg-white rounded-md outline outline-1 outline-gray-300 ring-1 ring-black ring-opacity-5 focus:outline-none"
          >
            <div class="flex flex-col space-y-4">
              <div class="flex flex-col">
                <label
                  for={`${props.id || "range"}-start-number`}
                  class="mb-2 text-sm font-medium text-gray-700"
                >
                  Greater than or equal to
                </label>
                <input
                  type="number"
                  id={`${props.id || "range"}-start-number`}
                  value={startNumber()}
                  onInput={(e) =>
                    setStartNumber(
                      e.target.value ? Number(e.target.value) : undefined
                    )
                  }
                  class="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-fuchsia-500 focus:border-fuchsia-500"
                  placeholder="Min value"
                />
              </div>
              <div class="flex flex-col">
                <label
                  for={`${props.id || "range"}-end-number`}
                  class="mb-2 text-sm font-medium text-gray-700"
                >
                  Less than
                </label>
                <input
                  type="number"
                  id={`${props.id || "range"}-end-number`}
                  value={endNumber()}
                  onInput={(e) =>
                    setEndNumber(
                      e.target.value ? Number(e.target.value) : undefined
                    )
                  }
                  class="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-fuchsia-500 focus:border-fuchsia-500"
                  placeholder="Max value"
                />
              </div>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default RangePicker;
