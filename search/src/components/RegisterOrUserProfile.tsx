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
import { For, Show, createEffect, createSignal } from "solid-js";
import {
  ClientEnvsConfiguration,
  isOrganizationDTO,
  isUserDTO,
  OrganizationDTO,
  type UserDTO,
} from "../../utils/apiTypes";
import { NotificationPopover } from "./Atoms/NotificationPopover";
import { AiFillGithub } from "solid-icons/ai";
import { TbMinusVertical } from "solid-icons/tb";
import { FaSolidCheck } from "solid-icons/fa";

export interface RegisterOrUserProfileProps {
  stars: number;
}

const RegisterOrUserProfile = (props: RegisterOrUserProfileProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const dataset = import.meta.env.PUBLIC_DATASET as string;
  const envs = JSON.parse(
    localStorage.getItem("clientConfig") ?? "{}",
  ) as ClientEnvsConfiguration;
  const showGithubStars = envs.PUBLIC_SHOW_GITHUB_STARS;

  const [isLoadingUser, setIsLoadingUser] = createSignal(true);
  const [currentUser, setCurrentUser] = createSignal<UserDTO | null>(null);
  const [currentOrganization, setOrganization] =
    createSignal<OrganizationDTO | null>(null);
  const [organizations, setOrganizations] = createSignal<
    OrganizationDTO[] | null
  >(null);

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
    const user = currentUser();
    if (!user) {
      return;
    }

    const orgItem = localStorage.getItem("currentOrganization");
    setOrganizations(user.orgs);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const organization = orgItem ? JSON.parse(orgItem) : null;
    if (organization && isOrganizationDTO(organization)) {
      // check if user is in the organization
      const org = user.orgs.find((o) => o.id === organization.id);
      if (org) {
        setOrganization(organization);
        return;
      }
    } else {
      setOrganization(user.orgs[0]);
    }
  });

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
          <Show when={!showGithubStars && props.stars}>
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
          <Show when={!!currentUser()}>
            <Popover defaultOpen={false} class="relative">
              {({ isOpen, setState }) => (
                <>
                  <PopoverButton
                    aria-label="Toggle filters"
                    type="button"
                    class="flex items-center space-x-1 pb-1 text-sm"
                  >
                    <span>{currentOrganization()?.name}</span>
                    <svg
                      fill="currentColor"
                      stroke-width="0"
                      style={{ overflow: "visible", color: "currentColor" }}
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 "
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
                      class="absolute right-0 z-10 mt-2 h-fit w-[180px] rounded-md border p-1 dark:bg-neutral-800"
                    >
                      <Menu class="mx-1 space-y-0.5">
                        <For each={organizations()}>
                          {(organizationItem) => {
                            const onClick = (e: Event) => {
                              e.preventDefault();
                              e.stopPropagation();
                              setOrganization(organizationItem);
                              localStorage.setItem(
                                "currentOrganization",
                                JSON.stringify(organizationItem),
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
                                    currentOrganization()?.id,
                                }}
                                onClick={onClick}
                              >
                                <div class="flex flex-row justify-start space-x-2">
                                  <span class="line-clamp-1 text-left text-sm">
                                    {organizationItem.name}
                                  </span>
                                </div>
                                {organizationItem.id ==
                                  currentOrganization()?.id && (
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
        </div>
      </Show>
    </div>
  );
};

export default RegisterOrUserProfile;
