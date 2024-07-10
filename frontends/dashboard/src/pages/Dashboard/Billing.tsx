import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { OrganizationAndSubAndPlan } from "shared/types";
import { PlansTable } from "../../components/PlansTable";
import { createToast } from "../../components/ShowToasts";
import { InvoicesTable } from "../../components/InvoicesTable";

export const Billing = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [orgSubPlan, setOrgSubPlan] =
    createSignal<OrganizationAndSubAndPlan | null>(null);

  const selectedOrganization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return (
      userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId) ?? null
    );
  });

  createEffect(() => {
    const selectedOrgId = selectedOrganization()?.id;
    if (!selectedOrgId) return;

    const orgSubPlanAbortController = new AbortController();
    void fetch(`${api_host}/organization/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
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
    <div class="flex w-full flex-col gap-y-12">
      <PlansTable currentOrgSubPlan={orgSubPlan()} />
      <InvoicesTable />
    </div>
  );
};
