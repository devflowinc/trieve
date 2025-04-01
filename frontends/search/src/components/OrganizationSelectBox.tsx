/* eslint-disable @typescript-eslint/unbound-method */
import { Show, For, createMemo, useContext, Switch, Match } from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { FaSolidCheck } from "solid-icons/fa";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";

export const OrganizationSelectBox = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $organizations = datasetAndUserContext.organizations;
  const organizationsList = createMemo(() => $organizations?.());
  const $currentOrganization = datasetAndUserContext.currentOrganization;

  return (
    <div>
      <div class="flex flex-col">
        <Popover defaultOpen={false} class="relative">
          {({ isOpen, setState }) => (
            <>
              <PopoverButton
                aria-label="Select Organization"
                type="button"
                class="flex items-center space-x-1 pb-1 text-sm"
              >
                <span class="line-clamp-1 text-left text-sm">
                  {$currentOrganization?.()?.name}
                </span>
                <Switch>
                  <Match when={isOpen()}>
                    <FiChevronUp class="h-3.5 w-3.5" />
                  </Match>
                  <Match when={!isOpen()}>
                    <FiChevronDown class="h-3.5 w-3.5" />
                  </Match>
                </Switch>
              </PopoverButton>
              <Show when={isOpen()}>
                <PopoverPanel
                  unmount={false}
                  class="absolute left-0 z-10 mt-2 h-fit w-[180px] rounded-md border bg-white p-1 dark:bg-neutral-800"
                >
                  <Menu class="mx-1 space-y-0.5">
                    <For each={organizationsList()}>
                      {(organizationItem) => {
                        const onClick = (e: Event) => {
                          e.preventDefault();
                          e.stopPropagation();
                          datasetAndUserContext.setCurrentOrganization(
                            organizationItem,
                          );
                          setState(false);
                        };
                        return (
                          <MenuItem
                            as="button"
                            classList={{
                              "flex w-full items-center justify-between rounded p-1 focus:text-black focus:outline-none dark:hover:text-white dark:focus:text-white hover:bg-neutral-300 hover:dark:bg-neutral-700":
                                true,
                              "bg-neutral-300 dark:bg-neutral-700":
                                organizationItem.id ===
                                $currentOrganization?.()?.id,
                            }}
                            onClick={onClick}
                          >
                            <div class="flex flex-row justify-start space-x-2">
                              <span class="line-clamp-1 text-left text-sm">
                                {organizationItem.name}
                              </span>
                            </div>
                            {organizationItem.id ==
                              $currentOrganization?.()?.id && (
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
      </div>
    </div>
  );
};
