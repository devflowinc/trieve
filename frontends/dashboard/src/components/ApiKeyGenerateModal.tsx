import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  Show,
  useContext,
} from "solid-js";
import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  DialogTitle,
  Disclosure,
  DisclosureButton,
  DisclosurePanel,
} from "terracotta";
import { SetUserApiKeyResponse } from "shared/types";
import { UserContext } from "../contexts/UserContext";
import { createToast } from "./ShowToasts";
import {
  FaRegularCircleQuestion,
  FaRegularClipboard,
  FaSolidChevronDown,
} from "solid-icons/fa";
import { Item, MultiSelect } from "./MultiSelect";
import { JsonInput, Tooltip } from "shared/ui";
import { ApiRoutes, RouteScope } from "./Routes";
import { z } from "zod";
import {
  chunkFilterSchema,
  searchMethodEnum,
  typoOptionsSchema,
} from "../analytics/utils/schemas/autocomplete";

export const ApiKeyGenerateModal = (props: {
  openModal: Accessor<boolean>;
  closeModal: () => void;
  onCreated: (api_key?: string) => void;
}) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [apiKey, setApiKey] = createSignal<string>("");
  const [name, setName] = createSignal<string>("");
  const [role, setRole] = createSignal<number>(0);
  const [generated, setGenerated] = createSignal<boolean>(false);
  const [selectedDatasetIds, setSelectedDatasetIds] = createSignal<Item[]>([]);
  const [selectedRoutes, setSelectedRoutes] = createSignal<Item[]>([]);
  const [defaultParams, setDefaultParams] =
    createSignal<Record<string, unknown>>();
  const [searchOptionsError, setSearchOptionsError] = createSignal<
    string | null
  >(null);
  const [expiresAt, setExpiresAt] = createSignal<string | null>(null);
  const availableRoutes = Object.keys(ApiRoutes).map((item, index) => ({
    id: `${index}`,
    name: item,
  }));

  const defaultParamsSchema = z
    .object({
      filters: chunkFilterSchema.nullish(),
      page_size: z.number().nullish(),
      remove_stop_words: z.boolean().nullish(),
      score_threshold: z.number().nullish(),
      search_type: searchMethodEnum.nullish(),
      slim_chunks: z.boolean().nullish(),
      typo_options: typoOptionsSchema.nullish(),
      use_quote_negated_terms: z.boolean().nullish(),
    })
    .strict();

  const datasetItems: Item[] =
    userContext.orgDatasets()?.map((dataset) => {
      return {
        id: dataset.dataset.id,
        name: dataset.dataset.name,
      };
    }) ?? [];

  const generateApiKey = () => {
    if (role() !== 0 && !role()) return;

    void fetch(`${apiHost}/organization/api_key`, {
      credentials: "include",
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": userContext.selectedOrg().id,
      },
      body: JSON.stringify({
        name: name(),
        role: role(),
        dataset_ids:
          selectedDatasetIds().length > 0
            ? selectedDatasetIds().map((d) => d.id)
            : undefined,
        scopes:
          selectedRoutes().length > 0
            ? selectedRoutes()
                .map((route) => ApiRoutes[route.name as RouteScope])
                .flat()
            : undefined,
        default_params: defaultParams(),
        expires_at: expiresAt() + " 00:00:00",
      }),
      // eslint-disable-next-line solid/reactivity
    }).then((res) => {
      if (res.ok) {
        // eslint-disable-next-line solid/reactivity
        void res.json().then((data) => {
          setApiKey((data as SetUserApiKeyResponse).api_key);
          props.onCreated(apiKey());
        });
        setGenerated(true);
      } else {
        createToast({ type: "error", title: "Failed to generate API key" });
      }
    });
  };

  createEffect((prevOpen) => {
    const curOpen = props.openModal();

    if (props.openModal() && !prevOpen) {
      setApiKey("");
      setName("");
      setRole(1);
      setGenerated(false);
    }

    return curOpen;
  }, false);

  const currentUserRole = createMemo(() => {
    const selectedOrgId = userContext.selectedOrg().id;
    if (!selectedOrgId) return 0;
    return (
      userContext
        .user?.()
        ?.user_orgs.find(
          (user_org) => user_org.organization_id === selectedOrgId,
        )?.role ?? 0
    );
  });

  return (
    <Show when={props.openModal()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-scroll"
        onClose={() => {
          props.closeModal();
          setGenerated(false);
          setApiKey("");
        }}
      >
        <div class="flex min-h-screen items-center justify-center px-4">
          <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />

          {/* This element is to trick the browser into centering the modal contents. */}
          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <DialogPanel class="my-8 inline-block w-full max-w-2xl transform rounded-md bg-white p-6 text-left align-middle shadow-xl transition-all">
            <Show when={!generated()}>
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  void generateApiKey();
                }}
              >
                <div class="space-y-12 sm:space-y-16">
                  <div>
                    <DialogTitle
                      as="h3"
                      class="text-base font-semibold leading-7"
                    >
                      Create New API Key
                    </DialogTitle>

                    <p class="mt-1 max-w-2xl text-sm leading-6 text-neutral-600">
                      You can use this API key to access your data from the API
                      by providing it in the Authorization header.
                    </p>

                    <div class="mt-2 space-y-8 border-b border-neutral-900/10 pb-12 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t sm:pb-0">
                      <div class="content-center sm:grid sm:grid-cols-3 sm:items-start sm:gap-4 sm:py-6">
                        <label
                          for="dataset-name"
                          class="block text-sm font-medium leading-6 sm:pt-1.5"
                        >
                          API Key Name
                        </label>
                        <div class="mt-2 sm:col-span-2 sm:mt-0">
                          <div class="flex rounded-md border border-neutral-300 sm:max-w-md">
                            <input
                              type="text"
                              name="dataset-name"
                              id="dataset-name"
                              autocomplete="dataset-name"
                              class="block flex-1 border-0 bg-transparent py-1.5 pl-1 placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm"
                              value={name()}
                              onInput={(e) => setName(e.currentTarget.value)}
                            />
                          </div>
                        </div>
                      </div>
                      <div class="sm:grid sm:grid-cols-3 sm:items-start sm:gap-4 sm:py-6">
                        <label
                          for="organization"
                          class="block text-sm font-medium leading-6 sm:pt-1.5"
                        >
                          Perm Level
                        </label>
                        <div class="mt-2 sm:col-span-2 sm:mt-0">
                          <select
                            id="location"
                            name="location"
                            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 text-sm focus:outline-magenta-500"
                            onSelect={(e) => {
                              setRole(parseInt(e.currentTarget.value));
                            }}
                            onChange={(e) => {
                              setRole(parseInt(e.currentTarget.value));
                            }}
                            value={role()}
                          >
                            <Show when={currentUserRole() >= 2}>
                              <option selected value={2}>
                                Owner
                              </option>
                            </Show>
                            <Show when={currentUserRole() >= 1}>
                              <option selected value={1}>
                                Admin
                              </option>
                            </Show>
                            <option value={0}>Read Only</option>
                          </select>
                        </div>
                      </div>
                      <Disclosure defaultOpen={false} as="div" class="py-2">
                        <DisclosureButton
                          as="div"
                          class="flex w-full justify-between rounded-l py-2 text-left text-sm focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75"
                        >
                          {({ isOpen }): JSX.Element => (
                            <>
                              <div class="flex items-center gap-x-2">
                                <span class="font-medium">API Key Scope</span>
                                <Tooltip
                                  body={<FaRegularCircleQuestion />}
                                  tooltipText="If not selected or empty, the API key will have access to all organizations and datasets that your current user profile has access to."
                                />
                              </div>
                              <FaSolidChevronDown
                                class={`${
                                  isOpen() ? "rotate-180 transform" : ""
                                } h-4 w-4`}
                                title={isOpen() ? "Close" : "Open"}
                              />
                            </>
                          )}
                        </DisclosureButton>
                        <DisclosurePanel class="space-y-2 pb-2 pt-1">
                          <div class="flex items-center space-x-2">
                            <label
                              for="organization"
                              class="block text-sm font-medium leading-6"
                            >
                              Expires:
                            </label>
                            <input
                              type="date"
                              class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                              value={expiresAt() ?? undefined}
                              onInput={(e) =>
                                setExpiresAt(e.currentTarget.value)
                              }
                            />
                          </div>
                          <div class="flex items-center space-x-2">
                            <label
                              for="organization"
                              class="block text-sm font-medium leading-6"
                            >
                              Datasets:
                            </label>
                            <MultiSelect
                              items={datasetItems}
                              selected={selectedDatasetIds()}
                              setSelected={(selected: Item[]) => {
                                setSelectedDatasetIds(selected);
                              }}
                            />
                          </div>
                          <div class="flex items-center space-x-2">
                            <label
                              for="organization"
                              class="block text-sm font-medium leading-6"
                            >
                              Routes:
                            </label>
                            <MultiSelect
                              items={availableRoutes}
                              selected={selectedRoutes()}
                              setSelected={(selected: Item[]) => {
                                setSelectedRoutes(selected);
                              }}
                            />
                          </div>
                        </DisclosurePanel>
                      </Disclosure>
                      <Disclosure defaultOpen={false} as="div" class="py-2">
                        <DisclosureButton
                          as="div"
                          class="flex w-full justify-between rounded-l py-2 text-left text-sm focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75"
                        >
                          {({ isOpen }): JSX.Element => (
                            <>
                              <div class="flex items-center gap-x-2">
                                <span class="font-medium">Default Params</span>
                                <Tooltip
                                  body={<FaRegularCircleQuestion />}
                                  tooltipText="You can set default params to be used with the API key. These params will be used as default values when the API key is used to access the API."
                                />
                              </div>
                              <FaSolidChevronDown
                                class={`${
                                  isOpen() ? "rotate-180 transform" : ""
                                } h-4 w-4`}
                                title={isOpen() ? "Close" : "Open"}
                              />
                            </>
                          )}
                        </DisclosureButton>
                        <DisclosurePanel class="space-y-2 pb-2 pt-1">
                          <JsonInput
                            onValueChange={(value) => {
                              const result =
                                defaultParamsSchema.safeParse(value);

                              if (result.success) {
                                setDefaultParams(result.data);
                                setSearchOptionsError(null);
                              } else {
                                setSearchOptionsError(
                                  result.error.errors.at(0)?.message ||
                                    "Invalid Search Options",
                                );
                              }
                            }}
                            value={() => {
                              return defaultParams()?.searchOptions;
                            }}
                            onError={(message) => {
                              setSearchOptionsError(message);
                            }}
                          />
                          <Show when={searchOptionsError()}>
                            <div class="text-red-500">
                              {searchOptionsError()}
                            </div>
                          </Show>
                        </DisclosurePanel>
                      </Disclosure>
                    </div>
                  </div>
                </div>

                <div class="mt-3 flex items-center justify-between">
                  <button
                    type="button"
                    class="rounded-md border px-2 py-1 text-sm font-semibold leading-6 hover:bg-neutral-50 focus:outline-magenta-500"
                    onClick={() => props.closeModal()}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={name() === ""}
                    class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm focus:outline-magenta-700 disabled:bg-magenta-200"
                  >
                    Generate New API Key
                  </button>
                </div>
              </form>
            </Show>
            <Show when={generated()}>
              <div class="mt-8">
                <div class="flex items-center justify-center">
                  <svg
                    class="h-12 w-12 text-green-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    aria-hidden="true"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                </div>
                <div class="mt-4 text-center">
                  <p class="text-neutral-600">
                    Here is your API Key. Make sure you copy this down as it
                    cannot be shown again:
                  </p>
                  <div class="mx-auto flex items-center justify-center gap-4 py-4">
                    <p class="font-semibold text-neutral-900">{apiKey()}</p>
                    <button
                      class="rounded border border-transparent p-2 hover:border-neutral-200 hover:bg-neutral-100"
                      onClick={() => {
                        void navigator.clipboard.writeText(apiKey());
                        window.dispatchEvent(
                          new CustomEvent("show-toast", {
                            detail: {
                              type: "info",
                              title: "Copied",
                              message: "API Key copied to clipboard",
                            },
                          }),
                        );
                      }}
                    >
                      <FaRegularClipboard />
                    </button>
                  </div>
                </div>
              </div>
              <button
                type="button"
                class="absolute left-0 top-0 m-2 rounded-full bg-white p-2 text-neutral-900 hover:bg-neutral-200 focus:outline-magenta-500"
                onClick={() => props.closeModal()}
              >
                <svg
                  class="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-hidden="true"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </Show>
          </DialogPanel>
        </div>
      </Dialog>
    </Show>
  );
};
