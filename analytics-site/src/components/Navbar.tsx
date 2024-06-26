import { Show, useContext, For } from "solid-js";
import { OrgContext } from "../contexts/OrgContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";
import { usePathname } from "../hooks/usePathname";
import { useBetterNav } from "../utils/useBetterNav";

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

export const Navbar = (props: NavbarProps) => {
  const userContext = useContext(UserContext);
  const orgContext = useContext(OrgContext);
  const pathname = usePathname();
  const navigate = useBetterNav();

  return (
    <div class="flex justify-between border border-b-neutral-400 bg-neutral-50 p-4">
      <div class="flex gap-3">
        <select
          onChange={(e) => {
            console.log(e.target.value);
            orgContext.selectOrg(e.currentTarget.value);
          }}
          value={orgContext.selectedOrg().id}
        >
          {
            <For each={userContext?.user().orgs}>
              {(org) => <option value={org.id}>{org.name}</option>}
            </For>
          }
        </select>

        <Show when={props.datasetOptions.length > 0}>
          <select
            onChange={(e) => {
              const dataset = props.datasetOptions.find(
                (dataset) => dataset.dataset.id === e.currentTarget.value,
              );
              if (dataset) {
                props.setSelectedDataset(dataset);
              }
            }}
            value={props.selectedDataset?.dataset.id}
          >
            <For each={props.datasetOptions}>
              {(dataset) => (
                <option value={dataset.dataset.id}>
                  {dataset.dataset.name}
                </option>
              )}
            </For>
          </select>
        </Show>
      </div>
      <div class="flex gap-4">
        <For each={navbarRoutes}>
          {(link) => {
            return (
              <div
                role="link"
                classList={{
                  "cursor-pointer": true,
                  "text-purple-800 underline": pathname() === link.href,
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
    </div>
  );
};
