import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { BiRegularLogIn, BiRegularLogOut, BiRegularUser } from "solid-icons/bi";
import { AiOutlineProfile } from "solid-icons/ai";
import { IoSettingsOutline } from "solid-icons/io";
import { Show, createEffect, createSignal } from "solid-js";
import { isUserDTO, type UserDTO } from "../../utils/apiTypes";
import { NotificationPopover } from "./Atoms/NotificationPopover";

const RegisterOrUserProfile = () => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;

  const [isLoadingUser, setIsLoadingUser] = createSignal(true);
  const [currentUser, setCurrentUser] = createSignal<UserDTO | null>(null);

  const logout = () => {
    void fetch(`${apiHost}/auth`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      if (!response.ok) {
        return;
      }
      window.location.href = "/";
    });
  };

  createEffect(() => {
    void fetch(`${apiHost}/auth`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isUserDTO(data)) {
            setCurrentUser(data);
            setIsLoadingUser(false);
          }
        });
        return;
      }
      setIsLoadingUser(false);
    });
  });

  return (
    <div>
      <Show when={!isLoadingUser()}>
        <div class="flex">
          <Show when={!currentUser()}>
            <div class="flex items-center space-x-2">
              <a href="/auth/login" class="min-[420px]:text-lg">
                Login
              </a>
              <a
                class="flex space-x-2 rounded-md bg-turquoise-500 p-2 text-neutral-900"
                href="/auth/register"
              >
                Register
                <BiRegularLogIn class="h-6 w-6" />
              </a>
            </div>
          </Show>
          <NotificationPopover user={currentUser()} />
          <Show when={!!currentUser()}>
            <Popover defaultOpen={false} class="relative flex items-center">
              {({ isOpen }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle user actions menu"
                    classList={{ flex: true }}
                  >
                    <BiRegularUser class="h-6 w-6 fill-current" />
                  </PopoverButton>
                  <Show when={isOpen()}>
                    <div>
                      <PopoverPanel
                        unmount={true}
                        class="absolute left-1/2 z-10 mt-5 -translate-x-[90%] transform px-4 sm:px-0"
                      >
                        <Menu class="flex flex-col space-y-1 overflow-hidden rounded-lg border border-slate-900 bg-neutral-100 p-1 shadow-lg drop-shadow-lg dark:bg-neutral-700 dark:text-white">
                          <MenuItem
                            class="h-0"
                            as="button"
                            aria-label="Empty"
                          />
                          <MenuItem
                            as="a"
                            class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:hover:bg-none dark:focus:bg-neutral-600"
                            href={`/user/${currentUser()?.id ?? ""}`}
                          >
                            <AiOutlineProfile class="h-6 w-6 fill-current" />
                            <div class="text-md font-medium">Profile</div>
                          </MenuItem>
                          <MenuItem
                            as="a"
                            class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:hover:bg-none dark:focus:bg-neutral-600"
                            href="/user/settings"
                          >
                            <IoSettingsOutline class="h-6 w-6 fill-current" />
                            <div class="text-md font-medium">Settings</div>
                          </MenuItem>
                          <MenuItem
                            as="button"
                            class="flex space-x-2 rounded-md px-2 py-1 hover:cursor-pointer focus:bg-neutral-100 focus:outline-none dark:hover:bg-neutral-600 dark:hover:bg-none dark:focus:bg-neutral-600"
                            onClick={logout}
                          >
                            <BiRegularLogOut class="h-6 w-6 fill-current" />
                            <div class="text-md font-medium">Logout</div>
                          </MenuItem>
                        </Menu>
                      </PopoverPanel>
                    </div>
                  </Show>
                </>
              )}
            </Popover>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default RegisterOrUserProfile;
