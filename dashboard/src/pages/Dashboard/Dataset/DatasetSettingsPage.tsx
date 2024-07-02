/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  For,
  Show,
  createEffect,
  createSignal,
  useContext,
  Switch,
  Match,
  createMemo,
} from "solid-js";
import { DatasetContext } from "../../../contexts/DatasetContext";
import {
  ServerEnvsConfiguration,
  availableEmbeddingModels,
} from "shared/types";
import { createToast } from "../../../components/ShowToasts";
import { AiOutlineInfoCircle } from "solid-icons/ai";
import { useNavigate } from "@solidjs/router";

export const defaultServerEnvsConfiguration: ServerEnvsConfiguration = {
  LLM_BASE_URL: "",
  LLM_DEFAULT_MODEL: "",
  EMBEDDING_BASE_URL: "https://embedding.trieve.ai",
  EMBEDDING_MODEL_NAME: "jina-base-en",
  MESSAGE_TO_QUERY_PROMPT: "",
  RAG_PROMPT: "",
  EMBEDDING_SIZE: 768,
  N_RETRIEVALS_TO_INCLUDE: 8,
  DUPLICATE_DISTANCE_THRESHOLD: 1.1,
  DOCUMENT_UPLOAD_FEATURE: true,
  DOCUMENT_DOWNLOAD_FEATURE: true,
  COLLISIONS_ENABLED: false,
  FULLTEXT_ENABLED: true,
  QDRANT_COLLECTION_NAME: null,
  EMBEDDING_QUERY_PREFIX: "Search for: ",
  USE_MESSAGE_TO_QUERY_PROMPT: false,
  FREQUENCY_PENALTY: null,
  TEMPERATURE: null,
  PRESENCE_PENALTY: null,
  STOP_TOKENS: null,
  INDEXED_ONLY: false,
  LOCKED: false,
};

export const ServerSettingsForm = () => {
  const datasetContext = useContext(DatasetContext);
  const [serverConfig, setServerConfig] = createSignal<ServerEnvsConfiguration>(
    datasetContext.dataset?.()?.server_configuration ??
      defaultServerEnvsConfiguration,
  );

  createEffect(() => {
    setServerConfig(
      datasetContext.dataset?.()?.server_configuration ??
        defaultServerEnvsConfiguration,
    );
  });

  const [saved, setSaved] = createSignal<boolean>(false);

  const onSave = () => {
    const datasetId = datasetContext.dataset?.()?.id;
    if (!datasetId) return;

    void fetch(`${import.meta.env.VITE_API_HOST}/dataset`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": datasetId,
      },
      credentials: "include",
      body: JSON.stringify({
        dataset_id: datasetContext.dataset?.()?.id,
        server_configuration: serverConfig(),
      }),
    })
      .then((resp) => {
        if (!resp.ok) {
          let message = "Error Saving Dataset Server Configuration";
          if (resp.status === 403) {
            message =
              "You must have owner permissions to modify dataset settings";
          }

          createToast({
            title: "Error",
            type: "error",
            message: message,
          });
        }

        setSaved(true);
        void new Promise((r) => setTimeout(r, 1000)).then(() =>
          setSaved(false),
        );
      })
      .catch((err) => {
        console.log(err);
      });
  };

  return (
    <form>
      <div class="rounded-md border shadow sm:overflow-hidden">
        <div class="rounded-md bg-white px-4 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-lg font-medium leading-6">
              Server Settings
            </h2>
            <p class="mt-1 text-sm text-neutral-600">
              Update settings for how the server behaves.
            </p>
          </div>

          <div class="mt-6 grid grid-cols-4 gap-6">
            <div class="col-span-4 sm:col-span-2">
              <label
                for="llmAPIURL"
                class="block text-sm font-medium leading-6"
              >
                LLM API URL
              </label>
              <input
                type="text"
                name="llmAPIURL"
                id="llmAPIURL"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().LLM_BASE_URL}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      LLM_BASE_URL: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="llmAPIURL"
                class="block text-sm font-medium leading-6"
              >
                LLM Default Model
              </label>
              <input
                type="text"
                name="llmDefaultModel"
                id="llmDefaultModel"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().LLM_DEFAULT_MODEL}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      LLM_DEFAULT_MODEL: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="nRetreivalsToInclude"
                class="block text-sm font-medium leading-6"
              >
                N Retrievals To Include (RAG-inference)
              </label>
              <input
                name="nRetreivalsToInclude"
                type="number"
                placeholder="something"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().N_RETRIEVALS_TO_INCLUDE}
                onChange={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      N_RETRIEVALS_TO_INCLUDE: parseFloat(
                        e.currentTarget.value,
                      ),
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="temperature"
                class="block text-sm font-medium leading-6"
              >
                Temperature (HyDE)
              </label>
              <input
                type="number"
                name="temperature"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().TEMPERATURE ?? 0}
                onChange={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      TEMPERATURE: e.currentTarget.valueAsNumber,
                    };
                  })
                }
              />
            </div>
            <div class="col-span-4 sm:col-span-2">
              <label
                for="presencePenalty"
                class="block text-sm font-medium leading-6"
              >
                Presence Penalty (HyDE)
              </label>
              <input
                type="number"
                name="presencePenalty"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().PRESENCE_PENALTY ?? 0}
                onChange={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      PRESENCE_PENALTY: e.currentTarget.valueAsNumber,
                    };
                  })
                }
              />
            </div>
            <div class="col-span-4 sm:col-span-2">
              <label
                for="frequencyPenalty"
                class="block text-sm font-medium leading-6"
              >
                Frequency Penalty (HyDE)
              </label>
              <input
                type="number"
                name="frequencyPenalty"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().FREQUENCY_PENALTY ?? 0}
                onChange={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      FREQUENCY_PENALTY: e.currentTarget.valueAsNumber,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="messageToQueryPrompt"
                class="block text-sm font-medium leading-6"
              >
                Message to Query Prompt (HyDE)
              </label>
              <textarea
                value={serverConfig().MESSAGE_TO_QUERY_PROMPT}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      MESSAGE_TO_QUERY_PROMPT: e.currentTarget.value,
                    };
                  })
                }
                rows="4"
                name="messageToQueryPrompt"
                id="messageToQueryPrompt"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="stopTokens"
                class="block text-sm font-medium leading-6"
              >
                Stop Tokens (HyDE)
              </label>
              <textarea
                value={serverConfig().STOP_TOKENS ?? ""}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      STOP_TOKENS: e.currentTarget.value,
                    };
                  })
                }
                rows="4"
                name="ragPrompt"
                id="ragPrompt"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>

            <div class="col-span-4 sm:col-span-2">
              <label
                for="ragPrompt"
                class="block text-sm font-medium leading-6"
              >
                RAG Prompt
              </label>
              <textarea
                value={serverConfig().RAG_PROMPT}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      RAG_PROMPT: e.currentTarget.value,
                    };
                  })
                }
                rows="4"
                name="ragPrompt"
                id="ragPrompt"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>

            <div class="col-span-4 space-y-1 sm:col-span-2">
              <div class="flex items-center">
                <label
                  for="embeddingSize"
                  class="mr-2 block text-sm font-medium leading-6"
                >
                  Embedding Model
                </label>
                <AiOutlineInfoCircle
                  class="h-5 w-5 text-neutral-400 hover:cursor-help"
                  title="Embedding Model is only editable on creation"
                />
              </div>
              <select
                id="embeddingSize"
                aria-readonly
                title="Embedding Model is only editable on creation"
                disabled
                name="embeddingSize"
                class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={
                  availableEmbeddingModels.find(
                    (model) => model.id === serverConfig().EMBEDDING_MODEL_NAME,
                  )?.name ?? availableEmbeddingModels[0].name
                }
              >
                <For each={availableEmbeddingModels}>
                  {(model) => <option value={model.name}>{model.name}</option>}
                </For>
              </select>
            </div>

            <div class="col-span-4 space-y-1 sm:col-span-2">
              <div class="flex items-center">
                <label
                  for="embeddingSize"
                  class="mr-2 block text-sm font-medium leading-6"
                >
                  Embedding Query Prefix
                </label>
                <AiOutlineInfoCircle
                  class="h-5 w-5 text-neutral-400 hover:cursor-help"
                  title="For some embedding models, the training data includes query prefixes. The default for Jina is 'Search for: '. You can experiment with different values."
                />
              </div>
              <input
                type="text"
                name="embeddingQueryPrefix"
                id="embeddingQueryPrefix"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={serverConfig().EMBEDDING_QUERY_PREFIX}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      EMBEDDING_QUERY_PREFIX: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="collisionsEnabled"
                id="collisionsEnabled"
                checked={serverConfig().USE_MESSAGE_TO_QUERY_PROMPT}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      USE_MESSAGE_TO_QUERY_PROMPT: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label
                for="collisionsEnabled"
                class="block text-sm font-medium leading-6"
              >
                Use Message to Query Prompt (HyDE)
              </label>
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="fullTextEnabled"
                id="fullTextEnabled"
                checked={serverConfig().FULLTEXT_ENABLED}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      FULLTEXT_ENABLED: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label
                for="fullTextEnabled"
                class="block text-sm font-medium leading-6"
              >
                Fulltext Enabled
              </label>
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="documentUploadFeature"
                id="documentUploadFeature"
                checked={serverConfig().DOCUMENT_UPLOAD_FEATURE}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      DOCUMENT_UPLOAD_FEATURE: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label
                for="documentUploadFeature"
                class="block text-sm font-medium"
              >
                Document upload feature
              </label>
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="lockDataset"
                id="lockDataset"
                checked={serverConfig().LOCKED}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      LOCKED: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label for="lockDataset" class="block text-sm font-medium">
                Lock dataset
              </label>
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="documentDownloadFeature"
                id="documentDownloadFeature"
                checked={serverConfig().DOCUMENT_DOWNLOAD_FEATURE}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      DOCUMENT_DOWNLOAD_FEATURE: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label
                for="documentDownloadFeature"
                class="block text-sm font-medium leading-6"
              >
                Document download feature
              </label>
            </div>
            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="indexedOnly"
                id="indexedOnly"
                checked={serverConfig().INDEXED_ONLY}
                onInput={(e) =>
                  setServerConfig((prev) => {
                    return {
                      ...prev,
                      INDEXED_ONLY: e.currentTarget.checked,
                    };
                  })
                }
              />
              <div class="flex items-center">
                <label
                  for="embeddingSize"
                  class="mr-2 block text-sm font-medium leading-6"
                >
                  Indexed Only
                </label>
                <AiOutlineInfoCircle
                  class="h-5 w-5 text-neutral-400 hover:cursor-help"
                  title="If enabled, only indexed documents will be returned in search results. This defaults to false because it can make it seem like ingested documents are missing because they are not yet indexed."
                />
              </div>
            </div>
          </div>
        </div>
        <div class="border-t bg-neutral-50 px-4 py-3 text-right sm:px-6">
          <button
            onClick={(e) => {
              e.preventDefault();
              onSave();
            }}
            class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600 disabled:bg-magenta-200"
          >
            Save
          </button>
          <Show when={saved()}>
            <span class="ml-3 text-sm">Saved!</span>
          </Show>
        </div>
      </div>
    </form>
  );
};

export const DangerZoneForm = () => {
  const datasetContext = useContext(DatasetContext);

  const navigate = useNavigate();

  const [deleting, setDeleting] = createSignal(false);
  const [confirmText, setConfirmText] = createSignal("");

  const deleteDataset = () => {
    const dataset_id = datasetContext.dataset?.()?.id;
    const organization_id = datasetContext.dataset?.()?.organization_id;
    if (!dataset_id) return;
    if (!organization_id) return;

    const confirmBox = confirm(
      "Deleting this dataset will remove all chunks which are contained within it. Are you sure you want to delete?",
    );
    if (!confirmBox) return;

    setDeleting(true);
    fetch(`${import.meta.env.VITE_API_HOST}/dataset`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": organization_id,
      },
      credentials: "include",
      body: JSON.stringify({
        dataset_id: dataset_id,
      }),
    })
      .then((res) => {
        setDeleting(false);
        if (res.ok) {
          navigate(`/dashboard/${organization_id}/overview`);
          createToast({
            title: "Success",
            message: "Dataset deleted successfully!",
            type: "success",
          });
        }
      })
      .catch(() => {
        setDeleting(false);
        createToast({
          title: "Error",
          message: "Error deleting dataset!",
          type: "error",
        });
      });
  };
  const datasetName = createMemo(() => datasetContext.dataset?.()?.name || "");

  return (
    <Show when={datasetContext.dataset != null}>
      <form class="rounded-md border border-red-600/20 shadow-sm shadow-red-500/30">
        <div class="shadow sm:overflow-hidden sm:rounded-md ">
          <div class="space-y-3 bg-white px-3 py-6 sm:p-6">
            <div>
              <h2 id="user-details-name" class="text-lg font-medium leading-6">
                Delete Dataset
              </h2>
              <p class="mt-0 text-sm text-red-700">
                Warning: This action is not reversible. Please be sure before
                deleting.
              </p>
              <div class="mt-3 grid grid-cols-4 gap-0">
                <div class="col-span-4 sm:col-span-2">
                  <label
                    for="dataset-name"
                    class="block text-sm font-medium leading-6 opacity-70"
                  >
                    Enter the dataset name
                    <span class="font-bold"> "{datasetName()}" </span>
                    to confirm.
                  </label>
                  <input
                    type="text"
                    name="dataset-name"
                    id="dataset-name"
                    class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                    value={confirmText()}
                    onInput={(e) => setConfirmText(e.currentTarget.value)}
                  />
                </div>
              </div>
            </div>
          </div>
          <div class="border-t border-red-600/20 bg-red-50/40 px-3 py-3 text-right sm:px-3">
            <button
              onClick={() => {
                deleteDataset();
              }}
              disabled={deleting() || confirmText() !== datasetName()}
              classList={{
                "pointer:cursor text-sm w-fit disabled:opacity-50 font-bold rounded-md bg-red-600/80 border px-4 py-2 text-white hover:bg-red-500 focus:outline-magenta-500":
                  true,
                "animate-pulse cursor-not-allowed": deleting(),
              }}
            >
              <Switch>
                <Match when={deleting()}>Deleting...</Match>
                <Match when={!deleting()}>Delete Dataset</Match>
              </Switch>
            </button>
          </div>
        </div>
      </form>
    </Show>
  );
};

export const DatasetSettingsPage = () => {
  return (
    <div class="flex flex-col gap-3 pb-4">
      <div>
        <ServerSettingsForm />
      </div>
      <div>
        <DangerZoneForm />
      </div>
    </div>
  );
};
