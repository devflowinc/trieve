import {
  For,
  Show,
  createMemo,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { FaRegularTrashCan } from "solid-icons/fa";
import { ApiKeyGenerateModal } from "./ApiKeyGenerateModal";
import { UserContext } from "../contexts/UserContext";
import {
  ApiKeyDTO,
  fromI32ToApiKeyRole,
  fromI32ToUserRole,
} from "../types/apiTypes";
import { formatDate } from "../formatters";

export const ApiKeys = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [openModal, setOpenModal] = createSignal<boolean>(false);

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
          return data as ApiKeyDTO[];
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
        <Show when={apiKeys().length === 0}>
          <div class="rounded-md border-[0.5px] border-neutral-300 bg-white py-4 text-center text-sm text-gray-500 shadow-sm">
            No API Keys
          </div>
        </Show>
        <Show when={apiKeys().length > 0}>
          <div class="inline-block min-w-full overflow-hidden rounded-md border-[0.5px] border-neutral-300 bg-white align-middle shadow-sm">
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
                          : fromI32ToApiKeyRole(apiKey.role).toString()}
                      </td>
                      <td class="px-3 py-3.5 text-center text-sm text-gray-900">
                        <div
                          onClick={(e) => {
                            e.preventDefault();
                            deleteApiKey(apiKey.id);
                          }}
                        >
                          <button>
                            <FaRegularTrashCan />
                          </button>
                        </div>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>
      <ApiKeyGenerateModal
        refetch={refetch}
        openModal={openModal}
        closeModal={() => setOpenModal(false)}
      />
    </>
  );
};
