import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { BiRegularLogOut, BiRegularUser } from "solid-icons/bi";
import { AiOutlineProfile } from "solid-icons/ai";
import { IoSettingsOutline } from "solid-icons/io";
import { Show, createEffect, createSignal } from "solid-js";
import { isUserDTO, type UserDTO } from "../../utils/apiTypes";
import { NotificationPopover } from "./Atoms/NotificationPopover";
import { AiFillGithub } from "solid-icons/ai";
import { TbMinusVertical } from "solid-icons/tb";

export interface RegisterOrUserProfileProps {
  stars: number;
}

const RegisterOrUserProfile = (props: RegisterOrUserProfileProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const dataset = import.meta.env.PUBLIC_DATASET as string;
  const showGithubStars = import.meta.env.PUBLIC_SHOW_GITHUB_STARS as string;

  const [isLoadingUser, setIsLoadingUser] = createSignal(true);
  const [currentUser, setCurrentUser] = createSignal<UserDTO | null>(null);

  const logout = () => {
    void fetch(`${apiHost}/auth`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": dataset,
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
    void fetch(`${apiHost}/auth/me`, {
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
        <div class="flex items-center space-x-2">
          <Show when={!currentUser()}>
            <div class="flex items-center space-x-3">
              <a
                href={`${apiHost}/auth?dataset_id=${dataset}`}
                class="min-[420px]:text-lg"
              >
                Login/Register
              </a>
            </div>
          </Show>
          <Show when={showGithubStars !== "off" && props.stars}>
            <a href="https://github.com/arguflow/arguflow">
              <div class="flex items-center justify-center rounded border border-black px-2 py-1 hover:border-gray-300 hover:bg-gray-300 dark:border-white dark:hover:border-neutral-700 dark:hover:bg-neutral-700">
                <AiFillGithub class="mr-2 h-[26px] w-[26px] fill-current" />
                <p class="text-sm">STAR US</p>
                <TbMinusVertical size={25} />
                <p>{props.stars}</p>
              </div>
            </a>
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
