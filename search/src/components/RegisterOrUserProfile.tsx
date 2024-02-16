import {
  Menu,
  MenuItem,
  Popover,
  PopoverButton,
  PopoverPanel,
} from "solid-headless";
import { BiRegularLogOut, BiRegularUser } from "solid-icons/bi";
import { IoSettingsOutline } from "solid-icons/io";
import { Show, createEffect, createSignal } from "solid-js";
import { AiFillGithub } from "solid-icons/ai";
import { TbMinusVertical } from "solid-icons/tb";
import { useStore } from "@nanostores/solid";
import { currentUser, isLoadingUser } from "../stores/userStore";
import { currentDataset } from "../stores/datasetStore";

const RegisterOrUserProfile = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;

  const $dataset = useStore(currentDataset);
  const $currentUser = useStore(currentUser);
  const $isLoadingUser = useStore(isLoadingUser);

  const [starCount, setStarCount] = createSignal(0);

  createEffect(() => {
    try {
      void fetch(`https://api.github.com/repos/devflowinc/trieve`, {
        headers: {
          Accept: "application/vnd.github+json",
        },
      }).then((response) => {
        if (!response.ok) {
          return;
        }
        void response.json().then((data) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          setStarCount(data.stargazers_count);
        });
      });
    } catch (e) {
      console.error(e);
    }
  });

  const logout = () => {
    const dataset = $dataset();
    if (!dataset) return;
    void fetch(`${apiHost}/auth`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
    }).then((response) => {
      if (!response.ok) {
        return;
      }
      window.location.href = "/";
    });
  };

  return (
    <div>
      <Show when={!$isLoadingUser()}>
        <div class="flex items-center space-x-2">
          <Show when={!$currentUser()}>
            <div class="flex items-center space-x-3">
              <a
                href={`${apiHost}/auth?redirect=http://localhost:5174`}
                class="min-[420px]:text-lg"
              >
                Login/Register
              </a>
            </div>
          </Show>
          <a href="https://github.com/devflowinc/trieve">
            <div class="flex items-center justify-center rounded border border-black px-2 py-1 hover:border-gray-300 hover:bg-gray-300 dark:border-white dark:hover:border-neutral-700 dark:hover:bg-neutral-700">
              <AiFillGithub class="mr-2 h-[26px] w-[26px] fill-current" />
              <p class="text-sm">STAR US</p>
              <TbMinusVertical size={25} />
              <p>{starCount()}</p>
            </div>
          </a>
          <Show when={!!$currentUser()}>
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
