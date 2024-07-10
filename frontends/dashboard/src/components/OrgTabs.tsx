import { A } from "@solidjs/router";
import { UserContext } from "../contexts/UserContext";
import { useContext } from "solid-js";

export const OrgTabs = () => {
  const userContext = useContext(UserContext);

  return (
    <div class="flex space-x-4">
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/overview`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Overview
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/users`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Users
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/billing`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Billing
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/api-keys`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        API Keys
      </A>
      <A
        href={`/dashboard/${userContext.selectedOrganizationId?.()}/settings`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
