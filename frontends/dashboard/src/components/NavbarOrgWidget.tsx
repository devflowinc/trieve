import { Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { FiChevronDown, FiLogOut, FiPlus, FiUser } from "solid-icons/fi";
import { Popover, PopoverButton, PopoverPanel, Transition } from "terracotta";
import { IconTypes } from "solid-icons";
import { OcPeople3 } from "solid-icons/oc";
import { useNavigate } from "@solidjs/router";

interface PopoutLinkProps {
  label: string;
  onClick: () => void;
  icon?: IconTypes;
}

export const NavbarOrgWidget = () => {
  const userInfo = useContext(UserContext);
  const navigate = useNavigate();

  const PopoutLink = (props: PopoutLinkProps) => {
    return (
      <button
        type="button"
        class="flex items-center gap-2 border-b border-b-neutral-300 p-1 px-2 text-sm font-medium last:border-b-transparent"
        onClick={props.onClick}
      >
        <Show when={props.icon}>{(icon) => icon()({})}</Show>
        <div>{props.label}</div>
      </button>
    );
  };

  return (
    <Popover class="relative" defaultOpen={false}>
      {({ isOpen }) => (
        <>
          <PopoverButton class="flex items-center gap-2 rounded-md border border-neutral-200 bg-neutral-100 p-1 px-2 text-sm">
            <FiUser class="text-neutral-500" />
            <div>{userInfo.user().email}</div>
            <FiChevronDown />
          </PopoverButton>
          <Show when={isOpen()}>
            <PopoverPanel class="absolute right-0 top-full z-10 mt-2 w-48 rounded-md border border-neutral-200 bg-white p-1 shadow-md">
              <div class="flex flex-col gap-2">
                <PopoutLink
                  label="Switch Organization"
                  onClick={() => userInfo.deselectOrg()}
                  icon={OcPeople3}
                />
                <PopoutLink
                  label="Create Organization"
                  onClick={() => userInfo.logout()}
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
  );
};
