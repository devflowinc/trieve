/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  For,
  Show,
  Accessor,
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
  availableDistanceMetrics,
  availableEmbeddingModels,
} from "shared/types";
import { createToast } from "../../../components/ShowToasts";
import { AiOutlineInfoCircle } from "solid-icons/ai";
import { useNavigate } from "@solidjs/router";
import { Tooltip } from "shared/ui";

const bm25Active = import.meta.env.VITE_BM25_ACTIVE as unknown as string;

export const defaultServerEnvsConfiguration: ServerEnvsConfiguration = {
  LLM_BASE_URL: "",
  LLM_DEFAULT_MODEL: "",
  LLM_API_KEY: "",
  EMBEDDING_BASE_URL: "https://embedding.trieve.ai",
  EMBEDDING_MODEL_NAME: "jina-base-en",
  MESSAGE_TO_QUERY_PROMPT: "",
  RAG_PROMPT: "",
  EMBEDDING_SIZE: 768,
  N_RETRIEVALS_TO_INCLUDE: 8,
  FULLTEXT_ENABLED: true,
  SEMANTIC_ENABLED: true,
  QDRANT_COLLECTION_NAME: null,
  EMBEDDING_QUERY_PREFIX: "Search for: ",
  USE_MESSAGE_TO_QUERY_PROMPT: false,
  FREQUENCY_PENALTY: null,
  TEMPERATURE: null,
  PRESENCE_PENALTY: null,
  STOP_TOKENS: null,
  MAX_TOKENS: null,
  INDEXED_ONLY: false,
  LOCKED: false,
  SYSTEM_PROMPT: null,
  MAX_LIMIT: 10000,
  BM25_ENABLED: bm25Active == "true",
  BM25_B: 0.75,
  BM25_K: 1.2,
  BM25_AVG_LEN: 256,
};

export const ServerSettingsForm = (props: {
  serverConfig: Accessor<ServerEnvsConfiguration>;
  setServerConfig: (
    config: (prev: ServerEnvsConfiguration) => ServerEnvsConfiguration,
  ) => void;
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
                  value={props.serverConfig().LLM_BASE_URL}
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
                  value={props.serverConfig().LLM_DEFAULT_MODEL}
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
                  value={props.serverConfig().MESSAGE_TO_QUERY_PROMPT}
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
                  value={props.serverConfig().STOP_TOKENS ?? ""}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
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
                  value={props.serverConfig().RAG_PROMPT}
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
                  checked={props.serverConfig().USE_MESSAGE_TO_QUERY_PROMPT}
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
                      checked={props.serverConfig().BM25_ENABLED}
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
                value={props.serverConfig().EMBEDDING_QUERY_PREFIX}
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
                value={props.serverConfig().N_RETRIEVALS_TO_INCLUDE}
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
                checked={props.serverConfig().SEMANTIC_ENABLED}
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
                checked={props.serverConfig().LOCKED}
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
                checked={props.serverConfig().FULLTEXT_ENABLED}
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
                checked={props.serverConfig().INDEXED_ONLY}
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
    fetch(`${import.meta.env.VITE_API_HOST}/dataset/${dataset_id}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset_id,
      },
      credentials: "include",
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
      <form
        class="rounded-md border border-red-600/20 shadow-sm shadow-red-500/30"
        id="danger-zone"
      >
        <div class="shadow sm:overflow-hidden sm:rounded-md">
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

export interface SaveButtonProps {
  onSave: () => void;
}

export const SaveButton = (props: SaveButtonProps) => {
  return (
    <div class="flex flex-col content-center justify-center space-y-3 border-t bg-neutral-50 px-1 py-3">
      <button
        onClick={(e) => {
          e.preventDefault();
          props.onSave();
        }}
        class="md:whitespace-wrap flex justify-center rounded-md bg-magenta-500 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-magenta-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600 disabled:bg-magenta-200"
      >
        Save Changes
      </button>
    </div>
  );
};

export const DatasetSettingsSidebar = (props: { onSave: () => void }) => {
  const [selectedDiv, setSelectedDiv] = createSignal<string>("");

  const handleClick = (event: MouseEvent) => {
    const target = event.currentTarget as HTMLAnchorElement;
    setSelectedDiv(target.getAttribute("href") || "");
  };

  return (
    <div class="sticky top-8">
      <nav class="space-y-4">
        <a
          href="#general-settings"
          onClick={handleClick}
          class={`block text-sm font-medium hover:text-magenta-500 ${
            selectedDiv() === "#general-settings"
              ? "text-magenta-500"
              : "text-gray-800"
          }`}
        >
          LLM Settings
        </a>
        <a
          href="#embedding-settings"
          onClick={handleClick}
          class={`block text-sm font-medium hover:text-magenta-500 ${
            selectedDiv() === "#embedding-settings"
              ? "text-magenta-500"
              : "text-gray-800"
          }`}
        >
          Embedding Settings
        </a>
        <a
          href="#additional-options"
          onClick={handleClick}
          class={`block text-sm font-medium hover:text-magenta-500 ${
            selectedDiv() === "#additional-options"
              ? "text-magenta-500"
              : "text-gray-800"
          }`}
        >
          Additional Options
        </a>
        <a
          href="#danger-zone"
          onClick={handleClick}
          class={`block text-sm font-medium hover:text-magenta-500 ${
            selectedDiv() === "#danger-zone"
              ? "text-magenta-500"
              : "text-gray-800"
          }`}
        >
          Dataset Management
        </a>
        <SaveButton onSave={props.onSave} />
      </nav>
    </div>
  );
};

export const DatasetSettingsPage = () => {
  const datasetContext = useContext(DatasetContext);

  const [originalConfig, setOriginalConfig] =
    createSignal<ServerEnvsConfiguration>(
      datasetContext.dataset?.()?.server_configuration ??
        defaultServerEnvsConfiguration,
    );

  const [serverConfig, setServerConfig] = createSignal<ServerEnvsConfiguration>(
    datasetContext.dataset?.()?.server_configuration ??
      defaultServerEnvsConfiguration,
  );

  createEffect(() => {
    const newConfig =
      datasetContext.dataset?.()?.server_configuration ??
      defaultServerEnvsConfiguration;
    setOriginalConfig(newConfig);
    setServerConfig(newConfig);
  });

  const getModifiedFields = () => {
    const modified: Partial<ServerEnvsConfiguration> = {};
    const original = originalConfig();
    const current = serverConfig();

    Object.keys(current).forEach((key) => {
      if (
        JSON.stringify(current[key as keyof ServerEnvsConfiguration]) !==
        JSON.stringify(original[key as keyof ServerEnvsConfiguration])
      ) {
        modified[key as keyof ServerEnvsConfiguration] = current[
          key as keyof ServerEnvsConfiguration
        ] as undefined;
      }
    });

    return modified;
  };

  const onSave = () => {
    const datasetId = datasetContext.dataset?.()?.id;
    if (!datasetId) return;

    const originalServerConfig = serverConfig();
    const modifiedFields = getModifiedFields();

    if (Object.keys(modifiedFields).length === 0) {
      createToast({
        title: "Info",
        type: "info",
        message: "No changes to save",
      });
      return;
    }

    void fetch(`${import.meta.env.VITE_API_HOST}/dataset`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": datasetId,
      },
      credentials: "include",
      body: JSON.stringify({
        dataset_id: datasetContext.dataset?.()?.id,
        server_configuration: modifiedFields,
      }),
    })
      .then((resp) => {
        if (resp.ok) {
          createToast({
            title: "Success",
            type: "success",
            message: "Dataset Configuration Saved",
          });
          setOriginalConfig(originalServerConfig);
          if (modifiedFields.LLM_API_KEY) {
            setServerConfig((prev) => ({ ...prev, LLM_API_KEY: "" }));
          }
          return;
        }

        if (!resp.ok) {
          let message = "Error Saving Dataset Configuration";
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
      })
      .catch((err) => {
        console.error(err);
      });
  };

  return (
    <div class="flex">
      <div class="flex w-5/6 flex-col gap-3 p-4 pb-4">
        <div>
          <ServerSettingsForm
            serverConfig={serverConfig}
            setServerConfig={setServerConfig}
          />
        </div>
        <div>
          <DangerZoneForm />
        </div>
      </div>
      <div class="w-1/6 p-6">
        <DatasetSettingsSidebar onSave={onSave} />
      </div>
    </div>
  );
};
