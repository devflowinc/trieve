import { Show, useContext, For, createMemo } from "solid-js";
import { OrgContext } from "../contexts/OrgContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";
import { usePathname } from "../hooks/usePathname";
import { useBetterNav } from "../utils/useBetterNav";
import { Select } from "shared/ui";
import {
  AiOutlineLineChart,
  AiOutlineTable,
  AiOutlineUser,
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

export const Sidebar = (props: NavbarProps) => {
  const userContext = useContext(UserContext);
  const orgContext = useContext(OrgContext);

  const pathname = usePathname();
  const navigate = useBetterNav();

  const dashboardUiURL = import.meta.env.VITE_DASHBOARD_UI_URL as string;

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

  //Construct a7b64c7f-01ad-43b2-aaaf-c78192ca3d72/start?org=ca34dafa-7826-41b4-9953-cd58617834f1
  const orgDatasetParams = createMemo(() => {
    const orgId = orgContext?.selectedOrg().id;
    const datasetId = props.selectedDataset?.dataset.id;
    let params = "";
    if (datasetId) params += datasetId;
    if (orgId && datasetId) params += `/start?org=${orgId}`;
    return params;
  });

  return (
    <div class="relative hidden h-screen flex-col justify-start overflow-y-auto border border-r-neutral-300 bg-neutral-100 p-4 lg:flex">
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
        <button type="button" class="hover:text-fuchsia-800">
          <a
            href={`${dashboardUiURL}/dashboard/dataset/${orgDatasetParams()}`}
            class="flex w-full flex-row items-center gap-2"
            target="_blank"
          >
            <TbLayoutDashboard />
            Dashboard
          </a>
        </button>
      </div>
      <div class="absolute bottom-0 left-0 right-0 flex flex-col items-start border-t border-t-neutral-300 bg-neutral-200/50 px-4 py-4">
        <div class="flex items-center gap-2">
          <p>{userContext?.user().email}</p>
          <AiOutlineUser class="h-4 w-4" />
        </div>
        <button
          class="flex items-center gap-2 hover:text-fuchsia-800"
          onClick={logOut}
        >
          Log Out <IoLogOutOutline class="inline-block h-4 w-4" />
        </button>
      </div>
    </div>
  );
};
