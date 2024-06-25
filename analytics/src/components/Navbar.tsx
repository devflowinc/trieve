import { Show, useContext } from "solid-js";
import { OrgContext } from "../contexts/OrgDatasetContext";
import { UserContext } from "../contexts/UserAuthContext";
import { DatasetAndUsage } from "shared/types";

interface NavbarProps {
  datasetOptions: DatasetAndUsage[];
  selectedDataset: DatasetAndUsage | null;
  setSelectedDataset: (dataset: DatasetAndUsage) => void;
}

export const Navbar = (props: NavbarProps) => {
  const userContext = useContext(UserContext);
  const orgContext = useContext(OrgContext);
  return (
    <div class="flex p-4 border border-b-neutral-400 bg-neutral-50">
      <div class="flex gap-3">
        <select
          onChange={(e) => {
            console.log(e.target.value);
            orgContext.selectOrg(e.currentTarget.value);
          }}
          value={orgContext.selectedOrg().id}
        >
          {userContext
            ?.user()
            .orgs?.map((org) => <option value={org.id}>{org.name}</option>)}
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
            {props.datasetOptions.map((dataset) => (
              <option value={dataset.dataset.id}>{dataset.dataset.name}</option>
            ))}
          </select>
        </Show>
      </div>
    </div>
  );
};
