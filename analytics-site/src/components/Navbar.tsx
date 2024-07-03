import { Show, useContext, For } from "solid-js";
import { OrgContext } from "../contexts/OrgContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";
import { usePathname } from "../hooks/usePathname";
import { useBetterNav } from "../utils/useBetterNav";
import { Select } from "shared/ui";

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
        <Select
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
              display={(dataset) => dataset.dataset.name}
              onSelected={(e) => {
                props.setSelectedDataset(e);
              }}
              options={props.datasetOptions}
              selected={selected()}
            />
          )}
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
