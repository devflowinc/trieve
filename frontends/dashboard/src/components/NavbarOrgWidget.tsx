import { createSignal, For, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { FiLogOut, FiPlus, FiUsers } from "solid-icons/fi";
import { Popover, PopoverButton, PopoverPanel } from "terracotta";
import { IconTypes } from "solid-icons";
import { OcPeople3 } from "solid-icons/oc";
import NewOrgModal from "./CreateNewOrgModal";
import { TbSelector } from "solid-icons/tb";
import { cn } from "shared/utils";

interface PopoutLinkProps {
  label: string;
  onClick: () => void;
  icon?: IconTypes;
}

export const NavbarOrgWidget = () => {
  const userInfo = useContext(UserContext);
  const [createOrgModalOpen, setCreateOrgModalOpen] = createSignal(false);

  const PopoutLink = (props: PopoutLinkProps) => {
    return (
      <button
        type="button"
        class="flex items-center gap-2 border-b border-b-neutral-300 p-1 px-2 text-sm font-medium last:border-b-transparent"
        onClick={() => props.onClick()}
      >
        <Show when={props.icon}>{(icon) => icon()({})}</Show>
        <div>{props.label}</div>
      </button>
    );
  };

  return (
    <>
      <Popover class="relative" defaultOpen={false}>
        {({ isOpen }) => (
          <>
            <PopoverButton class="flex items-center gap-2 rounded-md border border-neutral-300 p-1 px-2 text-sm">
              <FiUsers class="text-neutral-500" />
              <div>{userInfo.selectedOrg().name}</div>
              <TbSelector />
            </PopoverButton>
            <Show when={isOpen()}>
              <PopoverPanel class="absolute right-0 top-full z-10 mt-2 flex rounded-md border border-neutral-200 bg-white p-1 shadow-md">
                <OrgSelector />
                <div class="flex flex-col gap-2">
                  <PopoutLink
                    label="Switch Organization"
                    onClick={() => userInfo.deselectOrg()}
                    icon={OcPeople3}
                  />
                  <PopoutLink
                    label="Create Organization"
                    onClick={() => setCreateOrgModalOpen(true)}
                    icon={FiPlus}
                  />
                  <PopoutLink
                    label="Log Out"
                    onClick={() => userInfo.logout()}
                    icon={FiLogOut}
                  />
                </div>
              </PopoverPanel>
            </Show>
          </>
        )}
      </Popover>
      <NewOrgModal
        closeModal={() => setCreateOrgModalOpen(false)}
        isOpen={createOrgModalOpen}
      />
    </>
  );
};

const OrgSelector = () => {
  const orgContext = useContext(UserContext);

  return (
    <div class="min-w-[200px] border-r border-r-neutral-300">
      <div class="border-b border-b-neutral-200 pb-1 text-center font-medium">
        Switch Organizations
      </div>
      <div class="w-full">
        <For each={orgContext.user().orgs}>
          {(org) => (
            <button
              onClick={() => {
                orgContext.setSelectedOrg(org.id);
              }}
              class={cn(
                "relative flex w-full cursor-pointer items-center gap-2 p-2 px-4 text-left last:border-b-transparent hover:bg-neutral-100",
                org.id === orgContext.selectedOrg().id &&
                  "bg-magenta-300 text-white hover:bg-magenta-300",
              )}
            >
              <div class="absolute -left-1 bottom-0 top-0 w-1 bg-magenta-300" />
              <FiUsers
                size={12}
                class={cn(
                  "text-neutral-700",
                  org.id === orgContext.selectedOrg().id && "text-white",
                )}
              />
              <div class="text-sm font-medium">{org.name}</div>
              {/* <div class="text-[9px] text-neutral-500">{org.id}</div> */}
            </button>
          )}
        </For>
      </div>
    </div>
  );
};
