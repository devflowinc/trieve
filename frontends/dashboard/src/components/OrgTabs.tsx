import { A } from "@solidjs/router";
import { UserContext } from "../contexts/UserContext";
import { createMemo, useContext } from "solid-js";

export const OrgTabs = () => {
  const userContext = useContext(UserContext);

  const currentOrgId = createMemo(() => {
    return userContext.selectedOrganizationId?.();
  });

  return (
    <div class="flex space-x-4">
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/overview?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Overview
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/users?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Users
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/billing?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Billing
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/api-keys?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        API Keys
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/settings?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
