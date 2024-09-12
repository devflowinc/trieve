import { createMemo, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";

export const OrgName = () => {
  const userContext = useContext(UserContext);

  const selectedOrganization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  return (
    <h3 class="text-xl font-semibold text-neutral-600">
      {selectedOrganization()?.name} Organization
    </h3>
  );
};
