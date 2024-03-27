import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { OrganizationWithSubAndPlan } from "../../types/apiTypes";
import { PlansTable } from "../../components/PlansTable";

export const Billing = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [orgSubPlan, setOrgSubPlan] =
    createSignal<OrganizationWithSubAndPlan | null>(null);

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
      .then((res) => res.json())
      .then((data) => {
        setOrgSubPlan(data);
      });

    onCleanup(() => {
      orgSubPlanAbortController.abort();
    });
  });

  return (
    <div class="w-full pt-8">
      <PlansTable currentOrgSubPlan={orgSubPlan()} />
    </div>
  );
};
