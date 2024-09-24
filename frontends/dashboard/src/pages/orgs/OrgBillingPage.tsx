import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { OrganizationAndSubAndPlan } from "shared/types";
import { PlansTable } from "../../components/PlansTable";
import { createToast } from "../../components/ShowToasts";
import { InvoicesTable } from "../../components/InvoicesTable";
import { OrganizationUsageOverview } from "../../components/OrganizationUsageOverview";

export const OrgBillingPage = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [orgSubPlan, setOrgSubPlan] =
    createSignal<OrganizationAndSubAndPlan | null>(null);

  createEffect(() => {
    const orgSubPlanAbortController = new AbortController();
    void fetch(`${apiHost}/organization/${userContext.selectedOrg().id}`, {
      credentials: "include",
      headers: {
        "TR-Organization": userContext.selectedOrg().id,
      },
      signal: orgSubPlanAbortController.signal,
    })
      .then((res) => {
        if (res.status === 403) {
          createToast({
            title: "Error",
            type: "error",
            message:
              "It is likely that an admin or owner recently increased your role to admin or owner. Please sign out and sign back in to see the changes.",
            timeout: 10000,
          });
          return null;
        }

        return res.json();
      })
      .then((data) => {
        setOrgSubPlan(data);
      });

    onCleanup(() => {
      orgSubPlanAbortController.abort();
    });
  });

  return (
    <div class="pb-4">
      <OrganizationUsageOverview />
      <InvoicesTable />
      <PlansTable currentOrgSubPlan={orgSubPlan()} />
    </div>
  );
};
