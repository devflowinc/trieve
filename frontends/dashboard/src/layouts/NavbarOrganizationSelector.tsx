import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { FiUsers } from "solid-icons/fi";
import { Popover, PopoverButton, PopoverPanel } from "terracotta";
import { TbSelector } from "solid-icons/tb";
import { cn } from "shared/utils";
import { Event } from "shared/types";

const INTIAL_MAX_LENGTH = 4;

export const NavbarOrganizationSelector = () => {
  const userContext = useContext(UserContext);
  const [showingMoreOrgs, setShowingMoreOrgs] = createSignal(false);
  const [isOpen, setIsOpen] = createSignal(true);

  const canShowMoreOrgs = createMemo(() => {
    return (
      userContext.user().orgs.length > INTIAL_MAX_LENGTH && !showingMoreOrgs()
    );
  });

  const orgsToSelectFrom = createMemo(() => {
    const orgs = userContext.user().orgs;
    if (orgs.length > INTIAL_MAX_LENGTH && !showingMoreOrgs()) {
      // Reorder so the selected org is included
      const selectedOrg = orgs.find(
        (org) => org.id === userContext.selectedOrg().id,
      );
      if (selectedOrg) {
        orgs.splice(orgs.indexOf(selectedOrg), 1);
        orgs.unshift(selectedOrg);
      }
      return orgs.slice(0, INTIAL_MAX_LENGTH);
    }

    return orgs;
  });

  return (
    <div>
      <div>{JSON.stringify(isOpen())}</div>
      <Popover
        onClose={() => setIsOpen(false)}
        isOpen={isOpen()}
        class="relative"
      >
        {({ setState }) => (
          <>
            <PopoverButton
              onClick={() => {
                setIsOpen(!isOpen());
              }}
              class="flex items-center gap-2 rounded-md border border-neutral-300 p-1 px-2 text-sm"
            >
              <FiUsers class="text-neutral-500" />
              <div>{userContext.selectedOrg().name}</div>
              <TbSelector />
            </PopoverButton>
            <Show when={isOpen()}>
              <PopoverPanel class="absolute left-0 top-[100%] z-50 mt-2 w-auto min-w-[200px] rounded-md border border-neutral-200 bg-white shadow-md">
                <div>
                  {/* <For each={orgsToSelectFrom()}> */}
                  {/*   {(org) => ( */}
                  {/*     <button */}
                  {/*       onClick={() => { */}
                  {/*         // userContext.setSelectedOrg(org.id); */}
                  {/*       }} */}
                  {/*       class={cn( */}
                  {/*         "relative flex w-full cursor-pointer items-center gap-2 rounded-md p-2 px-4 text-left last:border-b-transparent hover:bg-neutral-100", */}
                  {/*         org.id === userContext.selectedOrg().id && */}
                  {/*           "bg-magenta-300 text-white hover:bg-magenta-300", */}
                  {/*       )} */}
                  {/*     > */}
                  {/*       <FiUsers */}
                  {/*         size={12} */}
                  {/*         class={cn( */}
                  {/*           "text-neutral-700", */}
                  {/*           org.id === userContext.selectedOrg().id && */}
                  {/*             "text-white", */}
                  {/*         )} */}
                  {/*       /> */}
                  {/*       <div class="text-sm font-medium">{org.name}</div> */}
                  {/*     </button> */}
                  {/*   )} */}
                  {/* </For> */}
                  <Show when={true}>
                    <button
                      onClick={(e: MouseEvent) => {
                        // e.stopPropagation();
                        setShowingMoreOrgs(true);
                      }}
                      class="w-full gap-2 rounded-md p-2 px-4 text-center text-sm"
                    >
                      Show More...
                    </button>
                  </Show>
                </div>
              </PopoverPanel>
            </Show>
          </>
        )}
      </Popover>
    </div>
  );
};
