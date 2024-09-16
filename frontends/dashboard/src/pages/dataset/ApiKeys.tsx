import {
  For,
  Show,
  createMemo,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { FaRegularTrashCan } from "solid-icons/fa";
import {
  ApiKeyRespBody,
  fromI32ToApiKeyRole,
  fromI32ToUserRole,
} from "shared/types";
import { UserContext } from "../../contexts/UserContext";
import { ApiKeyGenerateModal } from "../../components/ApiKeyGenerateModal";
import { formatDate } from "../../utils/formatters";
import { MagicSuspense } from "../../components/MagicBox";

export const ApiKeys = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [openModal, setOpenModal] = createSignal<boolean>(false);

  const currentUserRole = createMemo(() => {
    return (
      userContext
        .user()
        .user_orgs.find(
          (user_org) =>
            user_org.organization_id === userContext.selectedOrg().id,
        )?.role ?? 0
    );
  });

  const [apiKeys, { refetch, mutate }] = createResource(
    () => {
      return fetch(`${api_host}/user/api_key`, {
        method: "GET",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
      })
        .then((res) => res.json())
        .then((data) => {
          // Wait for 2 seonds
          return data as ApiKeyRespBody[];
        });
    },
    { initialValue: [] },
  );

  const deleteApiKey = (id: string) => {
    mutate((prev) => {
      return prev.filter((apiKey) => apiKey.id !== id);
    });

    void fetch(`${api_host}/user/api_key/${id}`, {
      method: "DELETE",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
    }).then((resp) => {
      if (resp.ok) {
        void refetch();
      }
    });
  };

  return (
    <>
      <div class="flex flex-col">
        <div class="flex items-end justify-between pb-2">
          <div class="text-lg font-medium">API Keys</div>
          <button
            type="button"
            class={
              "inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
            }
            onClick={(e) => {
              e.preventDefault();
              setOpenModal(true);
            }}
          >
            Create New Key +
          </button>
        </div>
        <MagicSuspense unstyled skeletonKey="apikeys">
          <Show when={apiKeys().length === 0}>
            <div class="rounded-md border-[0.5px] border-neutral-300 bg-white py-4 text-center text-sm text-gray-500 shadow-sm">
              No API Keys
            </div>
          </Show>
          <Show when={apiKeys().length > 0}>
            <div class="inline-block min-w-full overflow-hidden rounded-md border-[0.5px] border-neutral-300 bg-white align-middle shadow-md">
              <table class="min-w-full divide-y divide-gray-300">
                <thead class="w-full min-w-full bg-neutral-100">
                  <tr>
                    <th
                      scope="col"
                      class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                    >
                      Name
                    </th>
                    <th
                      scope="col"
                      class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                    >
                      Perms
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-left text-sm font-semibold"
                    >
                      Datasets
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-left text-sm font-semibold"
                    >
                      Organizations
                    </th>
                    <th
                      scope="col"
                      class="px-3 py-3.5 text-left text-sm font-semibold"
                    >
                      Created At
                    </th>
                    <th />
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 bg-white">
                  <For each={apiKeys()}>
                    {(apiKey) => (
                      <tr>
                        <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm text-gray-900 sm:pl-6 lg:pl-8">
                          {apiKey.name}
                        </td>
                        <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                          {apiKey.role > 0
                            ? fromI32ToUserRole(currentUserRole())
                            : fromI32ToApiKeyRole(apiKey.role).toString()}
                        </td>
                        <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                          <Show when={apiKey.dataset_ids?.length}>[</Show>
                          <For each={apiKey.dataset_ids}>
                            {(dataset_id, index) => (
                              <>
                                <a
                                  class="text-fuchsia-600 hover:underline"
                                  href={`/dashboard/dataset/${dataset_id}/start`}
                                >
                                  {dataset_id}
                                </a>
                                <Show
                                  when={
                                    index() <
                                    (apiKey.dataset_ids?.length ?? 0) - 1
                                  }
                                >
                                  {", "}
                                </Show>
                              </>
                            )}
                          </For>
                          <Show when={apiKey.dataset_ids?.length}>]</Show>
                        </td>
                        <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                          <Show when={apiKey.organization_ids?.length}>[</Show>
                          <For each={apiKey.organization_ids}>
                            {(org_id, index) => (
                              <>
                                <a
                                  class="text-fuchsia-600 hover:underline"
                                  href={`/dashboard/${org_id}/overview`}
                                  // onClick={() =>
                                  //   userContext.setSelectedOrganizationId(org_id)
                                  // }
                                >
                                  {org_id}
                                </a>
                                <Show
                                  when={
                                    index() <
                                    (apiKey.organization_ids?.length ?? 0) - 1
                                  }
                                >
                                  {", "}
                                </Show>
                              </>
                            )}
                          </For>
                          <Show when={apiKey.organization_ids?.length}>]</Show>
                        </td>
                        <td class="px-3 py-3.5 text-left text-sm text-gray-900">
                          {formatDate(new Date(apiKey.created_at))}
                        </td>
                        <td class="px-3 py-3.5 text-center text-sm text-gray-900">
                          <button
                            class="text-red-500 hover:text-neutral-900"
                            onClick={(e) => {
                              e.preventDefault();
                              confirm(
                                "Are you sure you want to delete this key?",
                              ) && deleteApiKey(apiKey.id);
                            }}
                          >
                            <FaRegularTrashCan />
                          </button>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </MagicSuspense>
      </div>
      <ApiKeyGenerateModal
        refetch={refetch}
        openModal={openModal}
        closeModal={() => setOpenModal(false)}
      />
    </>
  );
};
