import { Show, useContext, For } from "solid-js";
import { OrgContext } from "../contexts/OrgContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";
import { usePathname } from "../hooks/usePathname";
import { useBetterNav } from "../utils/useBetterNav";
import { Select } from "shared/ui";
import { AiOutlineUser } from "solid-icons/ai";
import { apiHost } from "../utils/apiHost";
import { IoLogOutOutline } from "solid-icons/io";
interface NavbarProps {
  datasetOptions: DatasetAndUsage[];
  selectedDataset: DatasetAndUsage | null;
  setSelectedDataset: (dataset: DatasetAndUsage) => void;
}

const navbarRoutes = [
  {
    href: "/",
    label: "Analytics",
  },
  {
    href: "/trends",
    label: "Trend Explorer",
  },
];

export const Sidebar = (props: NavbarProps) => {
  const userContext = useContext(UserContext);
  const orgContext = useContext(OrgContext);
  const pathname = usePathname();
  const navigate = useBetterNav();

  return (
    <div class="relative flex min-h-screen flex-col justify-start border border-r-neutral-300 bg-neutral-50 p-2 px-4 pr-8">
      <div class="items-center gap-3">
        <img
          class="h-12 w-12 cursor-pointer"
          src="https://cdn.trieve.ai/trieve-logo.png"
          alt="Logo"
        />
        <div class="h-4" />
        <Select
          label={<div class="text-sm opacity-60">Organization</div>}
          class="min-w-[150px]"
          display={(org) => org.name}
          onSelected={(e) => {
            console.log(e);
            orgContext.selectOrg(e.id);
          }}
          options={userContext?.user().orgs || []}
          selected={orgContext.selectedOrg()}
        />
        <Show when={props.datasetOptions.length > 0 && props.selectedDataset}>
          {(selected) => (
            <Select
              label={<div class="text-sm opacity-60">Dataset</div>}
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
      </div>
      <div class="items-center gap-4 pt-4">
        <For each={navbarRoutes}>
          {(link) => {
            return (
              <div
                role="link"
                classList={{
                  "cursor-pointer": true,
                  "text-purple-900 underline": pathname() === link.href,
                  "text-black": pathname() !== link.href,
                }}
                onClick={() => {
                  navigate(link.href);
                }}
              >
                {link.label}
              </div>
            );
          }}
        </For>
      </div>
      <div class="absolute bottom-0 left-0 right-0 flex flex-col items-start justify-self-end border-t px-4 py-4">
        <div class="flex items-center gap-2">
          <p>{userContext?.user().email}</p>
          <AiOutlineUser class="h-4 w-4" />
        </div>
        <button
          class="flex items-center gap-2 hover:text-fuchsia-800"
          onClick={() => {
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
                  console.log("error");
                });
            });
          }}
        >
          Log Out <IoLogOutOutline class="inline-block h-4 w-4" />
        </button>
      </div>
    </div>
  );
};
