import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import {
  DatasetAndUsage,
  OrganizationUsageCount,
  OrganizationWithSubAndPlan,
} from "../../types/apiTypes";
import NewDatasetModal from "../../components/NewDatasetModal";
import { DatasetOverview } from "../../components/DatasetOverview";
import { OrganizationUsageOverview } from "../../components/OrganizationUsageOverview";

export const Overview = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [datasetAndUsages, setDatasetsAndUsages] = createSignal<
    DatasetAndUsage[]
  >([]);
  const [orgSubPlan, setOrgSubPlan] =
    createSignal<OrganizationWithSubAndPlan>();
  const [orgUsage, setOrgUsage] = createSignal<OrganizationUsageCount>();
  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);

  const selectedOrganization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  createEffect(() => {
    const selectedOrgId = selectedOrganization()?.id;
    if (!selectedOrgId) return;

    const datasetAndUsageAbortController = new AbortController();
    void fetch(`${api_host}/dataset/organization/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
      },
      signal: datasetAndUsageAbortController.signal,
    })
      .then((res) => res.json())
      .then((data) => {
        setDatasetsAndUsages(data);
      });

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

    const orgUsageAbortController = new AbortController();
    void fetch(`${api_host}/organization/usage/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
      },
      signal: orgUsageAbortController.signal,
    })
      .then((res) => res.json())
      .then((data) => {
        setOrgUsage(data);
      });

    onCleanup(() => {
      datasetAndUsageAbortController.abort();
      orgSubPlanAbortController.abort();
      orgUsageAbortController.abort();
    });
  });

  return (
    <div class="pt-8">
      <OrganizationUsageOverview
        organization={orgSubPlan}
        orgUsage={orgUsage}
      />
      <DatasetOverview
        setOpenNewDatasetModal={setNewDatasetModalOpen}
        datasetAndUsages={datasetAndUsages}
        setDatasetsAndUsages={setDatasetsAndUsages}
      />
      <NewDatasetModal
        isOpen={newDatasetModalOpen}
        closeModal={() => {
          setNewDatasetModalOpen(false);
        }}
      />
    </div>
  );
};
