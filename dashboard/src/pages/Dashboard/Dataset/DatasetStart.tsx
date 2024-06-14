import { BiRegularInfoCircle, BiRegularLinkExternal } from "solid-icons/bi";
import CreateChunkRequest from "../../../components/CreateChunkRequest.md";
import HybridSearchReqeust from "../../../components/HybridSearchRequest.md";
import { BuildingSomething } from "../../../components/BuildingSomething";
import { createEffect, createMemo, createSignal, useContext } from "solid-js";
import { UserContext } from "../../../contexts/UserContext";
import { useLocation } from "@solidjs/router";
import { createToast } from "../../../components/ShowToasts";
import { Dataset } from "../../../types/apiTypes";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { FaRegularClipboard } from "solid-icons/fa";
import { AddSampleDataModal } from "../../../components/DatasetExampleModal";
import { BsMagic } from "solid-icons/bs";

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

  createEffect(() => {
    const pathname = location.pathname;
    const datasetId = pathname.split("/")[3];

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
            class="flex-col space-y-4 border bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
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
              <button
                class="flex items-center space-x-2 rounded-md border bg-magenta-500 px-2 py-1 text-sm text-white"
                onClick={() => setOpenSampleDataModal(true)}
              >
                <p>Add Sample Data</p>
                <BsMagic class="h-4 w-4" />
              </button>
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
              <p>1. Add searchable data</p>
              <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
                <div class="flex">
                  <div class="flex-shrink-0">
                    {/* <FiAlertTriangle class="h-4 w-4 text-yellow-400" /> */}
                    <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
                  </div>
                  <div class="ml-3">
                    <h3 class="text-sm font-semibold text-blue-800">
                      This example only uses 3 of 10 potential request
                      parameters
                    </h3>
                    <div class="mt-2 text-sm text-blue-700">
                      <p>
                        Read our{" "}
                        <a
                          href="https://redoc.trieve.ai/redoc#tag/chunk/operation/create_chunk"
                          class="underline"
                        >
                          OpenAPI docs for chunks
                        </a>{" "}
                        to see how to add metadata for filtering, a timestamp
                        for recency biasing, tags, and more.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              <div class="rounded-md border-[0.5px] border-neutral-300">
                <CreateChunkRequest />
              </div>
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
                      This example only uses 3 of 9 potential request parameters
                    </h3>
                    <div class="mt-2 text-sm text-blue-700">
                      <p>
                        Read our{" "}
                        <a
                          href="https://redoc.trieve.ai/redoc#tag/chunk/operation/search_chunk"
                          class="underline"
                        >
                          OpenAPI docs for search
                        </a>{" "}
                        to see how to add filters, manually weight semantic vs.
                        full-text importance, bias for recency, and more.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              <div class="rounded-md border-[0.5px] border-neutral-300">
                <HybridSearchReqeust />
              </div>
            </div>
          </section>
        </div>
      </main>
      <AddSampleDataModal
        openModal={openSampleDataModal}
        closeModal={() => setOpenSampleDataModal(false)}
      />
    </div>
  );
};
