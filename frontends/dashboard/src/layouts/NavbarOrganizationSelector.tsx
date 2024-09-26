import { createMemo, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { Select } from "shared/ui";
import { FiUsers } from "solid-icons/fi";

export const NavbarOrganizationSelector = () => {
  const userContext = useContext(UserContext);

  const organizationIds = createMemo(() =>
    userContext.user().orgs.map((org) => org.id),
  );

  const organizationNameFromId = (id: string) => {
    const organization = userContext.user().orgs.find((org) => org.id === id);
    if (!organization) {
      return "No Organization";
    }
    return organization.name;
  };

  return (
    <div>
      <Show when={organizationIds()}>
        {(organizations) => (
          <Select
            class="w-full bg-white"
            onSelected={userContext.setSelectedOrg}
            display={(id) => organizationNameFromId(id)}
            displayElement={(id) => (
              <div class="flex w-full items-center gap-2">
                <FiUsers />{" "}
                <div class="w-full text-sm">{organizationNameFromId(id)}</div>
              </div>
            )}
            selected={userContext.selectedOrg().id}
            options={organizations()}
          />
        )}
      </Show>
    </div>
  );
};
