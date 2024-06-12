import { FaSolidCheck } from "solid-icons/fa";
import { Show, For, useContext } from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "terracotta";
import { UserContext } from "../contexts/UserContext";

export const DatasetSelectBox = () => {
  const userContext = useContext(UserContext);

  return (
    <Show when={userContext.datasetsAndUsages?.()?.length != 0}>
      <Popover defaultOpen={false} class="relative">
        {({ isOpen, setState }) => (
          <>
            <PopoverButton
              aria-label="Toggle filters"
              type="button"
              class="flex items-center space-x-1 pb-1 text-sm"
            >
              <span class="line-clamp-1 text-left text-sm">
                {userContext.currentDataset?.()?.dataset?.name}
              </span>
              <svg
                fill="currentColor"
                stroke-width="0"
                style={{ overflow: "visible", color: "currentColor" }}
                viewBox="0 0 16 16"
                class="h-3.5 w-3.5"
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
                class="absolute bottom-5 left-0 z-20 mt-2 h-fit w-[180px] rounded-md border bg-white p-1 dark:bg-neutral-800"
              >
                <Menu class="mx-1 space-y-0.5">
                  <For each={userContext.datasetsAndUsages?.()}>
                    {(datasetItem) => {
                      const onClick = (e: Event) => {
                        e.preventDefault();
                        e.stopPropagation();
                        userContext.setCurrentDataset(datasetItem);
                        setState(false);
                      };
                      return (
                        <MenuItem
                          as="button"
                          classList={{
                            "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white hover:bg-neutral-300 hover:dark:bg-neutral-700":
                              true,
                            "bg-neutral-300 dark:bg-neutral-700":
                              datasetItem.dataset.id ===
                              userContext.currentDataset?.()?.dataset.id,
                          }}
                          onClick={onClick}
                        >
                          <div class="break-all px-1 text-left text-sm">
                            {datasetItem.dataset.name}
                          </div>
                          {datasetItem.dataset.id ==
                            userContext.currentDataset?.()?.dataset.id && (
                            <span>
                              <FaSolidCheck class="text-sm" />
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
    </Show>
  );
};
