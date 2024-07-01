import { BuildingSomething } from "../../../components/BuildingSomething";
import {
  createChunkRequest,
  hybridSearchRequest,
} from "../../../utils/createCodeSnippets";
import {
  Show,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { UserContext } from "../../../contexts/UserContext";
import { useLocation } from "@solidjs/router";
import { createToast } from "../../../components/ShowToasts";
import { Dataset, DatasetUsageCount, DefaultError } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { FaRegularClipboard } from "solid-icons/fa";
import { AiOutlineInfoCircle } from "solid-icons/ai";
import { TbReload } from "solid-icons/tb";
import { BiRegularInfoCircle, BiRegularLinkExternal } from "solid-icons/bi";
import { BsMagic } from "solid-icons/bs";
import { AddSampleDataModal } from "../../../components/DatasetExampleModal";
import { Tooltip } from "../../../components/Tooltip";
import { Codeblock } from "../../../components/Codeblock";

const SAMPLE_DATASET_SIZE = 921;

export const DatasetStart = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;
  const location = useLocation();
  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);

  const [openSampleDataModal, setOpenSampleDataModal] =
    createSignal<boolean>(false);

  const selectedOrganization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  const curDataset = createMemo(() => {
    const dataset = datasetContext.dataset?.();
    if (!dataset) return null;
    return dataset;
  });
  const currDatasetId = createMemo(() => curDataset()?.id);

  const [usage, { mutate: mutateUsage }] = createResource(
    currDatasetId,
    async (datasetId) => {
      const response = await fetch(`${api_host}/dataset/usage/${datasetId}`, {
        method: "GET",
        headers: {
          "TR-Dataset": datasetId,
          "Content-Type": "application/json",
        },
        credentials: "include",
      });

      if (response.ok) {
        const data = (await response.json()) as unknown as DatasetUsageCount;
        return data;
      } else {
        createToast({
          title: "Error",
          type: "error",
          message: "Failed to fetch dataset usage",
          timeout: 1000,
        });
        throw new Error("Failed to fetch dataset usage");
      }
    },
  );

  const reloadChunkCount = () => {
    const datasetId = currDatasetId();
    if (!datasetId) {
      console.error("Dataset ID is undefined.");
      return;
    }

    fetch(`${api_host}/dataset/usage/${datasetId}`, {
      method: "GET",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch dataset usage");
        }
        return response.json();
      })
      .then((newData) => {
        const currentUsage = usage();
        const prevCount = currentUsage?.chunk_count || 0;
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        const newCount: number = newData.chunk_count as number;
        const countDifference = newCount - prevCount;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        mutateUsage(() => newData);
        createToast({
          title: "Updated",
          type: "success",
          message: `Successfully updated chunk count: ${countDifference} chunk${
            Math.abs(countDifference) === 1 ? " has" : "s have"
          } been ${
            countDifference > 0
              ? "added"
              : countDifference < 0
                ? "removed"
                : "added or removed"
          } since last update.`,
          timeout: 3000,
        });
      })
      .catch((error) => {
        createToast({
          title: "Error",
          type: "error",
          message: `Failed to reload chunk count: ${error}`,
        });
      });
  };

  const [updatedTrackingId, setUpdatedTrackingId] = createSignal<
    string | undefined
  >(datasetContext.dataset?.()?.tracking_id);

  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const updateDataset = async () => {
    const organizationId = userContext.selectedOrganizationId?.();
    const dataset = datasetContext.dataset?.();
    if (!organizationId) return;
    if (!dataset) return;
    try {
      setIsLoading(true);
      const response = await fetch(`${api_host}/dataset`, {
        method: "PUT",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
          "TR-Organization": organizationId,
        },
        body: JSON.stringify({
          dataset_id: dataset.id,
          organization_id: organizationId,
          tracking_id: dataset.tracking_id,
          new_tracking_id: updatedTrackingId(),
          server_configuration: dataset.server_configuration,
          client_configuration: "{}",
        }),
      });
      if (!response.ok) {
        const error = (await response.json()) as DefaultError;
        throw new Error(error.message);
      }
      const newDataset = (await response.json()) as Dataset;
      createToast({
        title: "Success",
        type: "success",
        message: `New tracking id: ${newDataset.tracking_id}`,
      });
    } catch (e: unknown) {
      const error = e as Error;
      createToast({ title: "Error", type: "error", message: error.message });
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    const pathname = location.pathname;
    const datasetId = pathname.split("/")[3];

    if (!datasetId || !datasetId.match(/^[a-f0-9-]+$/)) {
      console.error("Invalid dataset id for fetch");
      return;
    }

    void fetch(`${api_host}/dataset/${datasetId}`, {
      method: "GET",
      headers: {
        "TR-Dataset": datasetId,
        "Content-Type": "application/json",
      },
      credentials: "include",
    }).then((resp) => {
      if (!resp.ok) {
        createToast({
          title: "Error",
          type: "error",
          message:
            "This dataset does not exist or do you not have permission to access it.",
          timeout: 1000,
        });
        return;
      }

      void resp.json().then((data: Dataset) => {
        userContext.setSelectedOrganizationId(data.organization_id);
      });
    });
  });

  return (
    <div class="h-full">
      <main class="mx-auto">
        <div class="space-y-6 pb-8 lg:grid lg:grid-cols-2 lg:gap-5 lg:px-0">
          <section
            class="flex-col space-y-4 border bg-white px-4 py-6 shadow sm:rounded-md sm:p-6 lg:col-span-2"
            aria-labelledby="organization-details-name"
          >
            <div class="flex items-center space-x-4">
              <h2 id="user-details-name" class="text-lg font-medium leading-6">
                Get Started
              </h2>
              <a
                class="flex items-center space-x-2 rounded-md border bg-neutral-100 px-2 py-1 text-sm"
                href="https://docs.trieve.ai"
                target="_blank"
              >
                <p>API Docs</p>
                <BiRegularLinkExternal class="h-4 w-4" />
              </a>
              <Show when={usage() && usage()?.chunk_count === 0}>
                <button
                  class="flex items-center space-x-2 rounded-md border bg-magenta-500 px-2 py-1 text-sm text-white"
                  onClick={() => setOpenSampleDataModal(true)}
                >
                  <p>Add Sample Data</p>
                  <BsMagic class="h-4 w-4" />
                </button>
              </Show>
            </div>
            <BuildingSomething />
            <div class="flex flex-col gap-2">
              <div class="flex items-center space-x-3">
                <p class="block text-sm font-medium">
                  {curDataset()?.name} dataset id:{" "}
                </p>
                <p class="w-fit text-sm">{curDataset()?.id}</p>
                <button
                  class="text-sm underline"
                  onClick={() => {
                    void navigator.clipboard.writeText(curDataset()?.id ?? "");
                    window.dispatchEvent(
                      new CustomEvent("show-toast", {
                        detail: {
                          type: "info",
                          title: "Copied",
                          message: "Dataset ID copied to clipboard",
                        },
                      }),
                    );
                  }}
                >
                  <FaRegularClipboard />
                </button>
              </div>
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
              <div class="flex items-center space-x-3">
                <p class="block text-sm font-medium">Chunk Count:</p>
                <p class="w-fit text-sm">{usage()?.chunk_count || 0}</p>
                <button
                  onClick={reloadChunkCount}
                  class="hover:text-fuchsia-500"
                >
                  <TbReload />
                </button>
              </div>
              <div class="flex items-center space-x-3">
                <label class="block text-sm font-medium">tracking id:</label>
                <div class="flex rounded-md border border-neutral-300 sm:max-w-md">
                  <input
                    type="text"
                    name="dataset-name"
                    id="dataset-name"
                    autocomplete="dataset-name"
                    class="block flex-1 border-0 bg-transparent py-1.5 pl-1 placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm"
                    value={datasetContext.dataset?.()?.tracking_id ?? ""}
                    onChange={(e) =>
                      setUpdatedTrackingId(e.currentTarget.value)
                    }
                  />
                </div>
                <button
                  disabled={isLoading()}
                  class="flex items-center gap-2 rounded-md border border-neutral-300 bg-white px-2 py-1.5 text-sm hover:border-fuchsia-800 hover:text-fuchsia-800"
                  onClick={() => void updateDataset()}
                >
                  Update
                </button>
                <Tooltip
                  body={<AiOutlineInfoCircle />}
                  tooltipText="Tracking ID can be used in TR-Dataset header for API requests"
                />
              </div>
            </div>
          </section>
          <section
            class="flex-col gap-4 border bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
            aria-labelledby="organization-details-name"
          >
            <h2 id="user-details-name" class="text-lg font-medium leading-6">
              Initial Request Examples
            </h2>
            <div class="flex flex-col space-y-4">
              <p>1. Add a searchable chunk</p>
              <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
                <div class="flex">
                  <div class="flex-shrink-0">
                    {/* <FiAlertTriangle class="h-4 w-4 text-yellow-400" /> */}
                    <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
                  </div>
                  <div class="ml-3">
                    <h3 class="text-sm font-semibold text-blue-800">
                      Create a chunk
                    </h3>
                    <div class="mt-2 text-sm text-blue-700">
                      <p>
                        Read our{" "}
                        <a
                          href="https://docs.trieve.ai/api-reference/chunk/create-or-upsert-chunk-or-chunks"
                          class="underline"
                        >
                          API reference for creating chunks
                        </a>{" "}
                        to see how to add tags and prices for filtering,
                        timestamps for recency biasing, and more.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              <Codeblock content={createChunkRequest(currDatasetId())} />
            </div>
            <div class="flex flex-col space-y-4">
              <p class="mt-3">2. Start Searching</p>
              <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
                <div class="flex">
                  <div class="flex-shrink-0">
                    {/* <FiAlertTriangle class="h-4 w-4 text-yellow-400" /> */}
                    <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
                  </div>
                  <div class="ml-3">
                    <h3 class="text-sm font-semibold text-blue-800">
                      Search chunks
                    </h3>
                    <div class="mt-2 text-sm text-blue-700">
                      <p>
                        Read our{" "}
                        <a
                          href="https://docs.trieve.ai/api-reference/chunk/search"
                          class="underline"
                        >
                          API reference for searching chunks
                        </a>{" "}
                        to see how to add filters, set highlight parameters,
                        bias for recency, and more.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              <Codeblock content={hybridSearchRequest(currDatasetId())} />
            </div>
          </section>
        </div>
      </main>
      <AddSampleDataModal
        addedDataCallback={() => {
          mutateUsage((prev) => {
            if (prev)
              return {
                ...prev,
                chunk_count: SAMPLE_DATASET_SIZE,
              };
          });
        }}
        openModal={openSampleDataModal}
        closeModal={() => setOpenSampleDataModal(false)}
      />
    </div>
  );
};
