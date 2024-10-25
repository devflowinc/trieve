/* eslint-disable @typescript-eslint/unbound-method */
import { Tooltip } from "shared/ui";
import { Button } from "solid-headless";
import { TbClick } from "solid-icons/tb";
import { createSignal, For } from "solid-js";
import { Popover, PopoverButton, PopoverPanel, Transition } from "terracotta";

type Props = {
  onSubmit: (eventType: string) => void;
};

export const CTRPopup = (props: Props) => {
  const [eventType, setEventType] = createSignal("click");
  return (
    <Popover defaultOpen={false} class="relative">
      {({ isOpen, close }) => (
        <>
          <Tooltip
            body={
              <PopoverButton>
                <TbClick class="h-5 w-5" />
              </PopoverButton>
            }
            tooltipText="Register event"
          />
          <Transition
            show={isOpen()}
            enter="transition duration-200"
            enterFrom="opacity-0 translate-y-1"
            enterTo="opacity-100 translate-y-0"
            leave="transition duration-150"
            leaveFrom="opacity-100 translate-y-0"
            leaveTo="opacity-0 translate-y-1"
          >
            <PopoverPanel
              unmount={false}
              class="absolute left-1/2 z-10 mt-3 w-auto -translate-x-1/2 transform px-4 sm:px-0"
            >
              <div class="flex w-full flex-col items-center gap-4 space-y-2 overflow-hidden rounded bg-white p-4 shadow-2xl dark:bg-shark-700">
                <div class="flex flex-col items-center gap-y-1">
                  <label aria-label="Change Filter Field">
                    <span class="p-1">Event Type:</span>
                  </label>
                  <select
                    class="h-fit w-auto rounded-md border border-neutral-400 bg-neutral-100 p-1 pl-1 dark:border-neutral-900 dark:bg-neutral-800"
                    onChange={(e) => setEventType(e.currentTarget.value)}
                    value={eventType()}
                  >
                    <For each={["view", "click", "add_to_cart", "purchase"]}>
                      {(eventType) => {
                        return (
                          <option class="flex w-full items-center justify-between rounded p-1">
                            {eventType}
                          </option>
                        );
                      }}
                    </For>
                  </select>
                </div>
                <Button
                  onClick={() => {
                    props.onSubmit(eventType());
                    close();
                  }}
                  class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
                >
                  Send Event
                </Button>
              </div>
            </PopoverPanel>
          </Transition>
        </>
      )}
    </Popover>
  );
};
