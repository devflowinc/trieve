import { For, Show, createSignal } from "solid-js";
import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import type { CardBookmarksDTO } from "../../utils/apiTypes";
import { VsOrganization } from "solid-icons/vs";

export interface CommunityBookmarkPopoverProps {
  bookmarks: CardBookmarksDTO[];
}

const CommunityBookmarkPopover = (props: CommunityBookmarkPopoverProps) => {
  const [usingPanel, setUsingPanel] = createSignal(false);

  return (
    <Popover defaultOpen={false} class="relative">
      {({ isOpen, setState }) => (
        <div>
          <div class="flex items-center">
            <PopoverButton title="Community Collections">
              <VsOrganization class="z-0 h-5 w-5 fill-current" />
            </PopoverButton>
          </div>
          <Show when={isOpen() || usingPanel()}>
            <PopoverPanel
              unmount={false}
              class="absolute z-50 w-screen max-w-xs -translate-x-[280px] translate-y-1"
              onMouseEnter={() => setUsingPanel(true)}
              onMouseLeave={() => setUsingPanel(false)}
              onClick={() => setState(true)}
            >
              <Menu class=" flex w-full flex-col justify-end space-y-2 overflow-hidden rounded bg-white py-4 shadow-xl dark:bg-shark-700">
                <div class="mb-3 w-full px-4 text-center text-lg font-bold">
                  Community Collections With This Card
                </div>
                <MenuItem as="button" aria-label="Empty" />
                <div class="scrollbar-track-rounded-md scrollbar-thumb-rounded-md max-w-screen mx-1 max-h-[20vh] transform justify-end space-y-2 overflow-y-auto rounded px-4 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-600 dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-400">
                  <For
                    each={props.bookmarks.flatMap((b) => b.slim_collections)}
                  >
                    {(collection, idx) => {
                      return (
                        <>
                          <Show when={idx() != 0}>
                            <div class="h-px w-full bg-neutral-200 dark:bg-neutral-700" />
                          </Show>
                          <div class="flex w-full items-center justify-between space-x-2">
                            <a
                              href={`/collection/${collection.id}`}
                              class="w-full underline"
                            >
                              {collection.name}
                            </a>
                          </div>
                        </>
                      );
                    }}
                  </For>
                </div>
              </Menu>
            </PopoverPanel>
          </Show>
        </div>
      )}
    </Popover>
  );
};

export default CommunityBookmarkPopover;
