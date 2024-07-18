import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import {
  OrganizationUsageCount,
  OrganizationAndSubAndPlan,
  Organization,
} from "shared/types";
import NewDatasetModal from "../../components/NewDatasetModal";
import { DatasetOverview } from "../../components/DatasetOverview";
import { OrganizationUsageOverview } from "../../components/OrganizationUsageOverview";
import { BiRegularLinkExternal } from "solid-icons/bi";
import { FaRegularClipboard } from "solid-icons/fa";
import { createToast } from "../../components/ShowToasts";
import { BuildingSomething } from "../../components/BuildingSomething";

export const Overview = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [orgSubPlan, setOrgSubPlan] = createSignal<OrganizationAndSubAndPlan>();
  const [orgUsage, setOrgUsage] = createSignal<OrganizationUsageCount>();
  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);

  const selectedOrganization = createMemo((): Organization | undefined => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return undefined;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  createEffect(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return;

    const orgSubPlanAbortController = new AbortController();
    void fetch(`${apiHost}/organization/${selectedOrgId}`, {
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
    void fetch(`${apiHost}/organization/usage/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
      },
      signal: orgUsageAbortController.signal,
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

          setOrgUsage({
            id: "",
            org_id: "",
            dataset_count: 0,
            user_count: 0,
            file_storage: 0,
            message_count: 0,
            chunk_count: 0,
          });
          return;
        }

        return res.json();
      })
      .then((data) => {
        setOrgUsage(data);
      })
      .catch((err) => {
        console.error(err);
      });

    onCleanup(() => {
      orgSubPlanAbortController.abort("cleanup");
      orgUsageAbortController.abort("cleanup");
    });
  });

  return (
    <div class="space-y-2 pb-8">
      <section
        class="mb-4 flex-col space-y-3 border bg-white py-4 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
        aria-labelledby="organization-details-name"
      >
        <div class="flex items-center space-x-4">
          <h2 id="user-details-name" class="text-lg font-medium leading-6">
            Create a Dataset Below to Get Started!
          </h2>
          <a
            class="flex items-center space-x-2 rounded-md border bg-neutral-100 px-2 py-1 text-sm"
            href="https://docs.trieve.ai"
            target="_blank"
          >
            <p>API Docs</p>
            <BiRegularLinkExternal class="h-4 w-4" />
          </a>
        </div>
        <BuildingSomething />
        <div class="flex flex-col space-y-2">
          <div class="flex items-center space-x-3">
            <p class="block text-sm font-medium">
              {selectedOrganization()?.name} org id:
            </p>
            <p class="w-fit text-sm">{selectedOrganization()?.id}</p>
            <button
              class="text-sm underline"
              onClick={() => {
                void navigator.clipboard.writeText(
                  selectedOrganization()?.id ?? "",
                );
                window.dispatchEvent(
                  new CustomEvent("show-toast", {
                    detail: {
                      type: "info",
                      title: "Copied",
                      message: "Organization ID copied to clipboard",
                    },
                  }),
                );
              }}
            >
              <FaRegularClipboard />
            </button>
          </div>
        </div>
      </section>

      <OrganizationUsageOverview
        organization={orgSubPlan}
        orgUsage={orgUsage}
      />
      <div class="h-1" />
      <Show when={selectedOrganization()}>
        <DatasetOverview
          selectedOrganization={selectedOrganization}
          setOpenNewDatasetModal={setNewDatasetModalOpen}
        />
      </Show>
      <NewDatasetModal
        isOpen={newDatasetModalOpen}
        closeModal={() => {
          setNewDatasetModalOpen(false);
        }}
      />
    </div>
  );
};
