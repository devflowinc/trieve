import { createMemo, Show, useContext, createEffect } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { DatasetContext } from "../contexts/DatasetContext";
import { Select } from "shared/ui";
import { FiUsers } from "solid-icons/fi";
import { useLocation } from "@solidjs/router";

export const NavbarOrganizationSelector = () => {
  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);
  const location = useLocation();

  const organizationIds = createMemo(
    () => userContext.user()?.orgs.map((org) => org.id),
  );

  const organizationNameFromId = (id: string) => {
    const organization = userContext.user()?.orgs.find((org) => org.id === id);
    return organization?.name;
  };

  createEffect(() => {
    const selectedOrg = userContext.selectedOrg();
    if (selectedOrg) {
      const orgDatasets = userContext.orgDatasets();
      if (
        orgDatasets &&
        orgDatasets.length > 0 &&
        !location.pathname.includes("/org") &&
        !orgDatasets.find(
          (orgDataset) => datasetContext.datasetId() == orgDataset.dataset.id,
        )
      ) {
        const firstDataset = orgDatasets[0].dataset;
        datasetContext.selectDataset(firstDataset.id);
      }
    }
  });

  return (
    <div>
      <Show when={organizationIds()}>
        {(organizations) => (
          <Select
            class="w-full bg-white"
            onSelected={userContext.setSelectedOrg}
            display={(id) => id}
            displayElement={(id) => (
              <div class="flex w-full items-center gap-2">
                <FiUsers />{" "}
                <div class="w-full text-sm">{organizationNameFromId(id)}</div>
              </div>
            )}
            selected={userContext.selectedOrg()?.id}
            options={organizations()}
          />
        )}
      </Show>
    </div>
  );
};
