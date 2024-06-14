import { Accessor, createMemo, createSignal, useContext, For } from "solid-js";
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  Transition,
  TransitionChild,
  DialogOverlay,
} from "terracotta";
import { UserContext } from "../contexts/UserContext";
import { useNavigate } from "@solidjs/router";
import {
  Dataset,
  DefaultError,
  ServerEnvsConfiguration,
  availableEmbeddingModels,
} from "../types/apiTypes";
import { defaultServerEnvsConfiguration } from "../pages/Dashboard/Dataset/DatasetSettingsPage";
import { createToast } from "./ShowToasts";

export interface NewDatasetModalProps {
  isOpen: Accessor<boolean>;
  closeModal: () => void;
}

export const NewExampleDatasetModal = (props: NewDatasetModalProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [serverConfig, setServerConfig] = createSignal<ServerEnvsConfiguration>(
    defaultServerEnvsConfiguration,
  );
  const userContext = useContext(UserContext);
  const [name, setName] = createSignal<string>("");
  const navigate = useNavigate();

  const selectedOrgnaization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  const createDataset = () => {
    const organizationId = userContext.selectedOrganizationId?.();
    if (!organizationId) return;

    const curServerConfig = serverConfig();

    fetch(`${apiHost}/dataset`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": organizationId,
      },
      body: JSON.stringify({
        dataset_name: name(),
        organization_id: organizationId,
        server_configuration: curServerConfig,
        client_configuration: "{}",
      }),
    })
      .then(async (res) => {
        if (res.ok) {
          return res.json();
        } else {
          const data = (await res.json()) as DefaultError;
          createToast({
            title: "Error",
            type: "error",
            message: data.message,
          });
        }
      })
      .then((res) => {
        if (!res) return;
        createToast({
          title: "Success",
          type: "success",
          message: "Successfully created dataset",
        });
        navigate(`/dashboard/dataset/${(res as Dataset).id}`);
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message: "There was an issue creating your dataset",
        });
      });
  };

  return (
    <Transition appear show={props.isOpen()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-auto"
        onClose={props.closeModal}
      >
        <div class="flex min-h-screen items-center justify-center px-4">
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />
          </TransitionChild>

          {/* This element is to trick the browser into centering the modal contents. */}
          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <DialogPanel class="inline-block w-full max-w-2xl transform overflow-hidden rounded-md bg-white p-6 text-left align-middle shadow-xl transition-all">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  createDataset();
                }}
              >
                <div class="space-y-12 sm:space-y-16">
                  <div>
                    <DialogTitle
                      as="h3"
                      class="text-base font-semibold leading-7"
                    >
                      Create Example Dataset
                    </DialogTitle>

                    <p class="max-w-2xl text-sm leading-6 text-neutral-600">
                      Your datset will be created and hosted on our servers with
                      data already in it.
                    </p>

                    <div class="mt-4 space-y-8 border-b border-neutral-900/10 pb-12 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t sm:pb-0">
                      <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                        <label
                          for="organization"
                          class="block h-full pt-1.5 text-sm font-medium leading-6"
                        >
                          Organization
                        </label>
                        <div class="mt-2 sm:col-span-2 sm:mt-0">
                          <select
                            id="location"
                            name="location"
                            class="block w-full select-none rounded-md border border-neutral-300 bg-white py-1.5 pl-2 pr-10 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                          >
                            <option>{selectedOrgnaization()?.name}</option>
                          </select>
                        </div>
                      </div>

                      <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                        <label
                          for="dataset-name"
                          class="block text-sm font-medium leading-6 sm:pt-1.5"
                        >
                          Dataset Name
                        </label>
                        <div class="mt-2 sm:col-span-2 sm:mt-0">
                          <div class="flex rounded-md border border-neutral-300 sm:max-w-md">
                            <span class="flex select-none items-center pl-3 text-neutral-600 sm:text-sm">
                              {selectedOrgnaization()?.name}/
                            </span>
                            <input
                              type="text"
                              name="dataset-name"
                              id="dataset-name"
                              autocomplete="dataset-name"
                              class="block flex-1 border-0 bg-transparent py-1.5 pl-1 placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm"
                              placeholder="my-dataset"
                              value={name()}
                              onInput={(e) => setName(e.currentTarget.value)}
                            />
                          </div>
                        </div>
                      </div>

                      <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                        <label
                          for="embeddingSize"
                          class="block h-full pt-1.5 text-sm font-medium leading-6"
                        >
                          Embedding Model
                        </label>
                        <select
                          id="embeddingSize"
                          name="embeddingSize"
                          class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                          value={
                            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
                            availableEmbeddingModels.find(
                              (model) =>
                                // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
                                model.id ===
                                serverConfig().EMBEDDING_MODEL_NAME,
                              // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
                            )?.name ?? availableEmbeddingModels[0].name
                          }
                          onChange={(e) => {
                            const selectedModel = availableEmbeddingModels.find(
                              (model) => model.name === e.currentTarget.value,
                            );

                            const embeddingSize =
                              selectedModel?.dimension ?? 1536;

                            setServerConfig((prev) => {
                              return {
                                ...prev,
                                EMBEDDING_SIZE: embeddingSize,
                                EMBEDDING_MODEL_NAME:
                                  selectedModel?.id ?? "jina-base-en",
                                EMBEDDING_QUERY_PREFIX:
                                  selectedModel?.id === "jina-base-en"
                                    ? "Search for:"
                                    : "",
                                EMBEDDING_BASE_URL:
                                  selectedModel?.url ??
                                  "https://api.openai.com/v1",
                              };
                            });
                          }}
                        >
                          <For each={availableEmbeddingModels}>
                            {(model) => (
                              <option value={model.name}>{model.name}</option>
                            )}
                          </For>
                        </select>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="mt-4 flex items-center justify-between">
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
                    Create Example Dataset
                  </button>
                </div>
              </form>
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
};
export default NewExampleDatasetModal;
