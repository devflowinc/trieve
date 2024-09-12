import { A, useParams } from "@solidjs/router";
import { createMemo, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";

export const DatasetTabs = () => {
  const userContext = useContext(UserContext);
  const urlParams = useParams();

  const currentOrgId = createMemo(() => {
    return userContext.selectedOrganizationId?.();
  });

  return (
    <div class="flex space-x-4">
      <A
        href={`/dashboard/dataset/${
          urlParams.dataset_id
        }/start?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Start
      </A>
      <A
        href={`/dashboard/dataset/${
          urlParams.dataset_id
        }/events?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Events
      </A>
      <A
        href={`/dashboard/dataset/${
          urlParams.dataset_id
        }/api-keys?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        API Keys
      </A>
      <A
        href={`/dashboard/dataset/${
          urlParams.dataset_id
        }/settings?org=${currentOrgId()}`}
        activeClass="border-b-2 -mb-[1px] border-magenta-500"
        class="hover:text-fuchsia-800"
      >
        Settings
      </A>
    </div>
  );
};
