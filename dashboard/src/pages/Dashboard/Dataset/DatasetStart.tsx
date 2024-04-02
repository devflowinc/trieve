import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  useContext,
} from "solid-js";
import {
  ApiKeyDTO,
  fromI32ToApiKeyRole,
  fromI32ToUserRole,
} from "../../../types/apiTypes";
import { UserContext } from "../../../contexts/UserContext";
import { BiRegularInfoCircle, BiRegularLinkExternal } from "solid-icons/bi";
import CreateChunkRequest from "../../../components/CreateChunkRequest.md";
import HybridSearchReqeust from "../../../components/HybridSearchRequest.md";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { ApiKeyGenerateModal } from "../../../components/ApiKeyGenerateModal";
import { FaRegularTrashCan } from "solid-icons/fa";
import { formatDate } from "../../../formatters";

export const DatasetStart = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);

  const [apiKeys, setApiKeys] = createSignal<ApiKeyDTO[]>([]);
  const [openModal, setOpenModal] = createSignal<boolean>(false);

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

  const currentUserRole = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return 0;
    return (
      userContext
        .user?.()
        ?.user_orgs.find(
          (user_org) => user_org.organization_id === selectedOrgId,
        )?.role ?? 0
    );
  });

  const getApiKeys = (abortController: AbortController) => {
    void fetch(`${api_host}/user/api_key`, {
      method: "GET",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      signal: abortController.signal,
    })
      .then((res) => res.json())
      .then((data) => {
        setApiKeys(data);
      });
  };

  const deleteApiKey = (id: string) => {
    void fetch(`${api_host}/user/api_key/${id}`, {
      method: "DELETE",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
    }).then((resp) => {
      if (resp.ok) {
        getApiKeys(new AbortController());
      }
    });
  };

  createEffect(
    on(openModal, () => {
      const abortController = new AbortController();

      getApiKeys(abortController);

      onCleanup(() => {
        abortController.abort();
      });
    }),
  );

  createEffect(() => {
    const abortController = new AbortController();

    getApiKeys(abortController);

    onCleanup(() => {
      abortController.abort();
    });
  });

  return (
    <div class="h-full">
      <main class="mx-auto">
        <div class="space-y-6 lg:grid lg:grid-cols-2 lg:gap-5 lg:px-0">
          <section
            class="flex-col space-y-4 bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
            aria-labelledby="organization-details-name"
          >
            <div class="flex items-center space-x-4">
              <h2 id="user-details-name" class="text-lg font-medium leading-6">
                Get Started
              </h2>
              <a
                class="flex items-center space-x-2 rounded-md bg-neutral-100 px-2 py-1 text-sm"
                href="https://docs.arguflow.ai"
                target="_blank"
              >
                <p>API Docs</p>
                <BiRegularLinkExternal class="h-4 w-4" />
              </a>
            </div>
            <div class="flex w-fit space-x-4 rounded-md bg-blue-50 px-6 py-4">
              <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
              <p class="text-sm text-blue-700">
                Building something? Call and tell us about it at{" "}
                <a class="underline" href="tel:6282224090">
                  +1 628-222-4090
                </a>{" "}
                or{" "}
                <a
                  class="underline"
                  href="https://matrix.to/#/#trieve-general:matrix.zerodao.gg"
                >
                  share in our Matrix
                </a>
                . We would love to hear about it and exchange tips!
              </p>
            </div>
            <div class="flex flex-col space-y-2">
              <div class="flex items-center space-x-3">
                <p class="block text-sm font-medium">
                  {selectedOrganization()?.name} org id:
                </p>
                <p class="text-sm">{selectedOrganization()?.id}</p>
              </div>
              <div class="flex items-center space-x-3">
                <p class="block text-sm font-medium">
                  {curDataset()?.name} dataset id:{" "}
                </p>
                <p class="text-sm">{curDataset()?.id}</p>
              </div>
              <div class="mt-6">
                <label
                  for="email-address"
                  class="mb-2 block text-sm font-medium"
                >
                  API Keys:
                </label>
                <Show when={apiKeys().length > 0}>
                  <div class="mb-1 mt-2">
                    <div class="inline-block min-w-full overflow-hidden rounded-md border-[0.5px] border-neutral-300 py-2 align-middle">
                      <table class="min-w-full divide-y divide-gray-300">
                        <thead>
                          <tr>
                            <th
                              scope="col"
                              class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6 lg:pl-8"
                            >
                              Name
                            </th>
                            <th
                              scope="col"
                              class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                            >
                              Created At
                            </th>
                            <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                              Perms
                            </th>
                          </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200 bg-white">
                          <For each={apiKeys()}>
                            {(apiKey) => (
                              <tr>
                                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm text-gray-900 sm:pl-6 lg:pl-8">
                                  {apiKey.name}
                                </td>
                                <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                                  {formatDate(new Date(apiKey.created_at))}
                                </td>
                                <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                                  {apiKey.role > 0
                                    ? fromI32ToUserRole(currentUserRole())
                                    : fromI32ToApiKeyRole(
                                        apiKey.role,
                                      ).toString()}
                                </td>
                                <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                                  <button>
                                    <FaRegularTrashCan
                                      onClick={(e) => {
                                        e.preventDefault();
                                        deleteApiKey(apiKey.id);
                                      }}
                                    />
                                  </button>
                                </td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                    </div>
                  </div>
                </Show>

                <button
                  type="button"
                  classList={{
                    "inline-flex mt-2 justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900":
                      true,
                  }}
                  onClick={(e) => {
                    e.preventDefault();
                    setOpenModal(true);
                  }}
                >
                  Create New API Key +
                </button>
              </div>
            </div>
          </section>
          <section
            class="flex-col space-y-4 bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
            aria-labelledby="organization-details-name"
          >
            <h2 id="user-details-name" class="text-lg font-medium leading-6">
              Initial Request Examples
            </h2>
            <div class="flex flex-col space-y-4">
              <p>1. Add searchable data</p>
              <div class="flex w-fit space-x-4 rounded-md bg-blue-50 px-6 py-4">
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
              <div class="flex w-fit space-x-4 rounded-md bg-blue-50 px-6 py-4">
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
      <ApiKeyGenerateModal
        openModal={openModal}
        closeModal={() => setOpenModal(false)}
      />
    </div>
  );
};
