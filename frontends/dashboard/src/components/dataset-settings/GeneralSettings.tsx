import { Tooltip } from "shared/ui";
import { AiOutlineInfoCircle } from "solid-icons/ai";
import { Show, For, Accessor } from "solid-js";
import { DatasetConfig } from "./LegacySettingsWrapper";
import {
  availableDistanceMetrics,
  availableEmbeddingModels,
} from "shared/types";

const bm25Active = import.meta.env.VITE_BM25_ACTIVE as unknown as string;

export const GeneralServerSettings = (props: {
  serverConfig: Accessor<DatasetConfig>;
  setServerConfig: (config: (prev: DatasetConfig) => DatasetConfig) => void;
}) => {
  return (
    <form class="flex flex-col gap-3">
      {/* General LLM Settings */}
      <div
        class="rounded-md border shadow sm:overflow-hidden"
        id="general-settings"
      >
        <div class="rounded-md bg-white px-4 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-xl font-medium leading-6">
              LLM Settings
            </h2>

            <p class="mt-1 text-sm text-neutral-600">
              Configure the general settings for the LLM.
            </p>
          </div>

          {/* General Settings */}
          <div class="mt-6">
            <span>
              <h2 class="text-lg font-semibold leading-6">General Settings</h2>
              <hr class="mt-2" />
            </span>

            <div class="mt-4 grid grid-cols-4 gap-6">
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
                  value={props.serverConfig().LLM_BASE_URL?.toString()}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        LLM_BASE_URL: e.currentTarget.value,
                      };
                    })
                  }
                />
              </div>

              <div class="col-span-4 sm:col-span-2">
                <div class="flex items-center">
                  <label
                    for="llmAPIURL"
                    class="block pr-2 text-sm font-medium leading-6"
                  >
                    LLM API Key
                  </label>
                  <Tooltip
                    direction="right"
                    body={<AiOutlineInfoCircle />}
                    tooltipText="LLM API Key cannot be viewed after you set it"
                  />
                </div>
                <input
                  type="text"
                  name="llmAPIURL"
                  id="llmAPIURL"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  value={props.serverConfig().LLM_API_KEY ?? ""}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        LLM_API_KEY: e.currentTarget.value,
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
                  value={props.serverConfig().LLM_DEFAULT_MODEL?.toString()}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        LLM_DEFAULT_MODEL: e.currentTarget.value,
                      };
                    })
                  }
                />
              </div>
            </div>
          </div>

          {/* Penalties and Parameters */}
          <div class="mt-6">
            <span>
              <h2 class="text-lg font-semibold leading-6">
                Penalties and Parameters
              </h2>
              <hr class="mt-2" />
            </span>

            <div class="mt-4 grid grid-cols-4 gap-6">
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
                  value={props.serverConfig().TEMPERATURE ?? 0}
                  onChange={(e) =>
                    props.setServerConfig((prev) => {
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
                  value={props.serverConfig().PRESENCE_PENALTY ?? 0}
                  onChange={(e) =>
                    props.setServerConfig((prev) => {
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
                  value={props.serverConfig().FREQUENCY_PENALTY ?? 0}
                  onChange={(e) =>
                    props.setServerConfig((prev) => {
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
                  for="presencePenalty"
                  class="block text-sm font-medium leading-6"
                >
                  Max Tokens
                </label>
                <input
                  type="number"
                  name="presencePenalty"
                  id="linesBeforeShowMore"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  value={props.serverConfig().MAX_TOKENS ?? 0}
                  onChange={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        MAX_TOKENS: e.currentTarget.valueAsNumber,
                      };
                    })
                  }
                />
              </div>
            </div>
          </div>

          {/* Prompt Settings */}
          <div class="mt-6">
            <span>
              <h2 class="text-lg font-semibold leading-6">Prompt Settings</h2>
              <hr class="mt-2" />
            </span>

            <div class="mt-4 grid grid-cols-4 gap-6">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="messageToQueryPrompt"
                  class="block text-sm font-medium leading-6"
                >
                  Message to Query Prompt (HyDE)
                </label>
                <textarea
                  value={
                    props.serverConfig().MESSAGE_TO_QUERY_PROMPT || undefined
                  }
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
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
                  value={props.serverConfig().STOP_TOKENS?.join(",") ?? ""}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        STOP_TOKENS: e.currentTarget.value.split(","),
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
                  for="messageToQueryPrompt"
                  class="block text-sm font-medium leading-6"
                >
                  System Prompt
                </label>
                <textarea
                  value={props.serverConfig().SYSTEM_PROMPT ?? ""}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      return {
                        ...prev,
                        SYSTEM_PROMPT: e.currentTarget.value,
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
                  for="ragPrompt"
                  class="block text-sm font-medium leading-6"
                >
                  RAG Prompt
                </label>
                <textarea
                  value={props.serverConfig().RAG_PROMPT || ""}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
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

              <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
                <input
                  type="checkbox"
                  name="collisionsEnabled"
                  id="collisionsEnabled"
                  checked={
                    props.serverConfig().USE_MESSAGE_TO_QUERY_PROMPT ?? false
                  }
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
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
            </div>
          </div>

          {/* bm25 Settings */}
          <Show when={bm25Active == "true"}>
            <div class="mt-6">
              <span>
                <h2 class="text-lg font-semibold leading-6">bm25 Settings</h2>
                <hr class="mt-2" />
              </span>

              <div class="mt-4 grid grid-cols-4 gap-6">
                <Show when={bm25Active == "true"}>
                  <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
                    <input
                      type="checkbox"
                      name="bm25TextEnabled"
                      id="bm25TextEnabled"
                      checked={!!props.serverConfig().BM25_ENABLED}
                      onInput={(e) =>
                        props.setServerConfig((prev) => {
                          return {
                            ...prev,
                            BM25_ENABLED: e.currentTarget.checked,
                          };
                        })
                      }
                    />
                    <label
                      for="bm25TextEnabled"
                      class="block text-sm font-medium leading-6"
                    >
                      BM25 Enabled
                    </label>
                  </div>
                </Show>

                <Show when={props.serverConfig().BM25_ENABLED}>
                  <div class="col-span-4 sm:col-span-2">
                    <label
                      for="bm25B"
                      class="block text-sm font-medium leading-6"
                    >
                      BM25 B Parameter
                    </label>
                    <input
                      type="number"
                      name="bm25B"
                      id="bm25B"
                      class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                      value={props.serverConfig().BM25_B ?? 0}
                      onChange={(e) =>
                        props.setServerConfig((prev) => {
                          return {
                            ...prev,
                            BM25_B: e.currentTarget.valueAsNumber,
                          };
                        })
                      }
                    />
                  </div>
                </Show>

                <Show when={props.serverConfig().BM25_ENABLED}>
                  <div class="col-span-4 sm:col-span-2">
                    <label
                      for="bm25K"
                      class="block text-sm font-medium leading-6"
                    >
                      BM25 K Parameter
                    </label>
                    <input
                      type="number"
                      name="bm25K"
                      id="bm25K"
                      class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                      value={props.serverConfig().BM25_K?.toFixed(2) ?? 0}
                      onChange={(e) =>
                        props.setServerConfig((prev) => {
                          return {
                            ...prev,
                            BM25_K: e.currentTarget.valueAsNumber,
                          };
                        })
                      }
                    />
                  </div>
                </Show>

                <Show when={props.serverConfig().BM25_ENABLED}>
                  <div class="col-span-4 sm:col-span-2">
                    <label
                      for="bm25Length"
                      class="block text-sm font-medium leading-6"
                    >
                      BM25 Average Chunk Length
                    </label>
                    <input
                      type="number"
                      name="bm25Length"
                      id="bm25Length"
                      class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                      value={props.serverConfig().BM25_AVG_LEN ?? 0}
                      onChange={(e) =>
                        props.setServerConfig((prev) => {
                          return {
                            ...prev,
                            BM25_AVG_LEN: e.currentTarget.valueAsNumber,
                          };
                        })
                      }
                    />
                  </div>
                </Show>
              </div>
            </div>
          </Show>
        </div>
      </div>

      {/* Embedding Settings */}
      <div
        class="rounded-md border shadow sm:overflow-hidden"
        id="embedding-settings"
      >
        <div class="rounded-md bg-white px-4 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-xl font-medium leading-6">
              Embedding Settings
            </h2>
            <p class="mt-1 text-sm text-neutral-600">
              Configure the embedding model and query parameters.
            </p>
          </div>

          <div class="mt-6 grid grid-cols-4 gap-6">
            <div class="col-span-4 space-y-1 sm:col-span-2">
              <div class="flex items-center">
                <label
                  for="embeddingSize"
                  class="mr-2 block text-sm font-medium leading-6"
                >
                  Embedding Model
                </label>
                <Tooltip
                  direction="right"
                  body={<AiOutlineInfoCircle />}
                  tooltipText="Embedding Model is only editable on creation"
                />
              </div>
              <select
                id="embeddingSize"
                aria-readonly
                title="Embedding Model is only editable on creation"
                disabled
                name="embeddingSize"
                class="col-span-2 block w-full cursor-not-allowed rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={
                  availableEmbeddingModels.find(
                    (model) =>
                      model.id === props.serverConfig().EMBEDDING_MODEL_NAME,
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
                <Tooltip
                  direction="right"
                  body={<AiOutlineInfoCircle />}
                  tooltipText="For some embedding models, the training data includes query prefixes. The default for Jina is 'Search for: '. You can experiment with different values."
                />
              </div>
              <input
                type="text"
                name="embeddingQueryPrefix"
                id="embeddingQueryPrefix"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={props.serverConfig().EMBEDDING_QUERY_PREFIX || ""}
                onInput={(e) =>
                  props.setServerConfig((prev) => {
                    return {
                      ...prev,
                      EMBEDDING_QUERY_PREFIX: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 space-y-1 sm:col-span-2">
              <div class="flex items-center">
                <label
                  for="embeddingSize"
                  class="mr-2 block text-sm font-medium leading-6"
                >
                  Distance Metric
                </label>
                <Tooltip
                  direction="right"
                  body={<AiOutlineInfoCircle />}
                  tooltipText="Distance Metric is only editable on creation"
                />
              </div>
              <select
                id="embeddingSize"
                aria-readonly
                title="Embedding Model is only editable on creation"
                disabled
                name="embeddingSize"
                class="col-span-2 block w-full cursor-not-allowed rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={
                  availableDistanceMetrics.find(
                    (metric) =>
                      metric.id === props.serverConfig().DISTANCE_METRIC,
                  )?.name ?? availableEmbeddingModels[0].name
                }
              >
                <For each={availableDistanceMetrics}>
                  {(metric) => (
                    <option value={metric.name}>{metric.name}</option>
                  )}
                </For>
              </select>
            </div>
          </div>
        </div>
      </div>

      {/* Additional Options */}
      <div
        class="rounded-md border shadow sm:overflow-hidden"
        id="additional-options"
      >
        <div class="rounded-md bg-white px-4 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-xl font-medium leading-6">
              Additional Options
            </h2>
            <p class="mt-1 text-sm text-neutral-600">
              Fine-tune server and model settings.
            </p>
          </div>

          <div class="mt-6 grid grid-cols-4 gap-6">
            <div class="col-span-4 sm:col-span-2">
              <label
                for="nRetreivalsToInclude"
                class="block text-sm font-medium leading-6"
              >
                Documents to include for RAG
              </label>
              <input
                name="nRetreivalsToInclude"
                type="number"
                placeholder="something"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={props.serverConfig().N_RETRIEVALS_TO_INCLUDE?.toString()}
                onChange={(e) =>
                  props.setServerConfig((prev) => {
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
              <div class="flex flex-row items-center gap-2">
                <label
                  for="maxLimit"
                  class="block text-sm font-medium leading-6"
                >
                  Max Count Limit
                </label>
                <Tooltip
                  direction="right"
                  body={<AiOutlineInfoCircle />}
                  tooltipText="Sets the maximum limit when counting chunks, applies to search and count routes in the API to prevent DDOS attacks on the server."
                />
              </div>
              <input
                type="number"
                name="maxLimit"
                id="linesBeforeShowMore"
                class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={props.serverConfig().MAX_LIMIT ?? 0}
                onChange={(e) =>
                  props.setServerConfig((prev) => {
                    return {
                      ...prev,
                      MAX_LIMIT: e.currentTarget.valueAsNumber,
                    };
                  })
                }
              />
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="semanticEnabled"
                id="semanticEnabled"
                checked={!!props.serverConfig().SEMANTIC_ENABLED}
                onInput={(e) =>
                  props.setServerConfig((prev) => {
                    return {
                      ...prev,
                      SEMANTIC_ENABLED: e.currentTarget.checked,
                    };
                  })
                }
              />
              <label
                for="semanticEnabled"
                class="block text-sm font-medium leading-6"
              >
                Semantic Enabled
              </label>
            </div>

            <div class="col-span-4 flex items-center space-x-2 sm:col-span-2">
              <input
                type="checkbox"
                name="lockDataset"
                id="lockDataset"
                checked={!!props.serverConfig().LOCKED}
                onInput={(e) =>
                  props.setServerConfig((prev) => {
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
                name="fullTextEnabled"
                id="fullTextEnabled"
                checked={!!props.serverConfig().FULLTEXT_ENABLED}
                onInput={(e) =>
                  props.setServerConfig((prev) => {
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
                name="indexedOnly"
                id="indexedOnly"
                checked={!!props.serverConfig().INDEXED_ONLY}
                onInput={(e) =>
                  props.setServerConfig((prev) => {
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
                <Tooltip
                  direction="right"
                  body={<AiOutlineInfoCircle />}
                  tooltipText="If enabled, only indexed documents will be returned in search results."
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </form>
  );
};
