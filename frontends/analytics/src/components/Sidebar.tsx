import { Show, useContext, For, createMemo } from "solid-js";
import { OrgContext } from "../contexts/OrgContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";
import { usePathname } from "../hooks/usePathname";
import { useBetterNav } from "../utils/useBetterNav";
import { Select, Tooltip } from "shared/ui";
import {
  AiOutlineLineChart,
  AiOutlineTable,
  AiOutlineUser,
  AiOutlineApi,
} from "solid-icons/ai";
import { apiHost } from "../utils/apiHost";
import { IoChatboxOutline, IoLogOutOutline } from "solid-icons/io";
import { HiOutlineMagnifyingGlass, HiOutlineNewspaper } from "solid-icons/hi";
import { TbLayoutDashboard } from "solid-icons/tb";

interface NavbarProps {
  datasetOptions: DatasetAndUsage[];
  selectedDataset: DatasetAndUsage | null;
  setSelectedDataset: (dataset: DatasetAndUsage) => void;
}

const navbarRoutes = [
  {
    href: "/",
    label: "Overview",
    icon: HiOutlineNewspaper,
  },
  {
    href: "/analytics",
    label: "Search Analytics",
    icon: AiOutlineLineChart,
  },
  {
    href: "/rag",
    label: "RAG Analytics",
    icon: IoChatboxOutline,
  },
  {
    href: "/trends",
    label: "Trend Explorer",
    icon: HiOutlineMagnifyingGlass,
  },
  {
    href: "/data/searches",
    label: "Data Explorer",
    icon: AiOutlineTable,
  },
];

const dashboardURL = import.meta.env.VITE_DASHBOARD_URL as string;
const searchUrl = import.meta.env.VITE_SEARCH_UI_URL as string;
const chatUrl = import.meta.env.VITE_CHAT_UI_URL as string;

export const Sidebar = (props: NavbarProps) => {
  const userContext = useContext(UserContext);
  const orgContext = useContext(OrgContext);

  const domainNavbarRoutes = createMemo(() => {
    const domainNavbarRoutes = [
      {
        href: `${dashboardURL}/dashboard/dataset/${props.selectedDataset
          ?.dataset.id}/start?org=${orgContext.selectedOrg().id}`,
        label: "Dashboard",
        icon: TbLayoutDashboard,
      },
      {
        href: "https://docs.trieve.ai/api-reference/",
        label: "API Docs",
        icon: AiOutlineApi,
      },
      {
        href: `${searchUrl}?organization=${
          orgContext.selectedOrg().id
        }&dataset=${props.selectedDataset?.dataset.id}`,
        label: "Search Playground",
        icon: HiOutlineMagnifyingGlass,
      },
      {
        href: `${chatUrl}?organization=${
          orgContext.selectedOrg().id
        }&dataset=${props.selectedDataset?.dataset.id}`,
        label: "Chat Playground",
        icon: IoChatboxOutline,
      },
    ];
    return domainNavbarRoutes;
  });

  const pathname = usePathname();
  const navigate = useBetterNav();

  const logOut = () => {
    void fetch(`${apiHost}/auth?redirect_uri=${window.origin}`, {
      method: "DELETE",
      credentials: "include",
    }).then((res) => {
      res
        .json()
        .then((res) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
          window.location.href = res.logout_url;
        })
        .catch(() => {
          console.error("error");
        });
    });
  };

  return (
    <div class="relative hidden h-screen flex-col justify-start overflow-y-auto border border-r-neutral-300 bg-neutral-100 lg:flex">
      <div class="flex flex-grow flex-col p-4">
        <div class="flex items-center gap-1">
          <img
            class="h-12 w-12 cursor-pointer"
            src="https://cdn.trieve.ai/trieve-logo.png"
            alt="Logo"
          />
          <div>
            <div class="text-2xl font-semibold leading-none">Trieve</div>
            <div class="pl-1 text-sm leading-tight text-neutral-600">
              Analytics
            </div>
          </div>
        </div>
        <div class="border-neutral-20 h-4 border-b" />
        <div>
          <Select
            label={<div class="pt-2 text-sm opacity-60">Organization</div>}
            class="min-w-[220px]"
            options={userContext?.user().orgs || []}
            display={(org) => org.name}
            onSelected={(e) => {
              orgContext.selectOrg(e.id);
            }}
            selected={orgContext.selectedOrg()}
            id="dataset-select"
          />
        </div>
        <Show when={props.datasetOptions.length > 0 && props.selectedDataset}>
          {(selected) => (
            <Select
              label={<div class="pt-2 text-sm opacity-60">Dataset</div>}
              class="min-w-[220px]"
              options={props.datasetOptions}
              display={(dataset) => dataset.dataset.name}
              onSelected={(e) => {
                props.setSelectedDataset(e);
              }}
              selected={selected()}
              id="dataset-select"
            />
          )}
        </Show>
        <div class="border-neutral-20 h-4 border-b" />
        <div class="flex flex-grow flex-col justify-between">
          <div class="flex flex-col gap-4 px-2 pt-4">
            <For each={navbarRoutes}>
              {(link) => {
                return (
                  <div
                    role="link"
                    classList={{
                      "cursor-pointer flex items-center gap-2": true,
                      "text-purple-900 underline": pathname() === link.href,
                      "text-black": pathname() !== link.href,
                    }}
                    onClick={() => {
                      navigate(link.href);
                    }}
                  >
                    {link.icon({ size: "18px" })}
                    {link.label}
                  </div>
                );
              }}
            </For>
          </div>
          <div>
            <div class="h-4 border-b border-neutral-400/50" />

            <div class="flex flex-col gap-2 px-2 pt-4">
              <For each={domainNavbarRoutes()}>
                {(link) => {
                  return (
                    <a
                      role="link"
                      classList={{
                        "cursor-pointer flex items-center text-sm gap-2 hover:text-fuchsia-500":
                          true,
                        "text-purple-900 underline": pathname() === link.href,
                        "text-black": pathname() !== link.href,
                      }}
                      href={link.href}
                      target="_blank"
                    >
                      {link.icon({ size: "14px" })}
                      {link.label}
                    </a>
                  );
                }}
              </For>
            </div>
          </div>
        </div>
      </div>
      <div class="justify-self-end border-t border-t-neutral-300 bg-neutral-200/50">
        <div class="flex w-full items-center justify-between gap-2 px-6 py-4">
          <div class="flex items-center gap-2">
            <AiOutlineUser class="h-4 w-4" />
            <span>{userContext?.user().email}</span>
          </div>
          <Tooltip
            tooltipClass="text-sm border border-neutral-300 bg-neutral-200 min-w-max"
            unsetWidth
            tooltipText="Log Out"
            direction="top"
          >
            <button
              class="flex items-center gap-2 justify-self-end p-1 pl-2 opacity-60 hover:text-fuchsia-800"
              onClick={logOut}
            >
              <IoLogOutOutline class="inline-block h-4 w-4" />
            </button>
          </Tooltip>
        </div>
      </div>
    </div>
  );
};
