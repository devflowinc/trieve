import { Tooltip } from "shared/ui";
import { AiOutlineInfoCircle } from "solid-icons/ai";
import { Accessor, createEffect, onCleanup } from "solid-js";
import { DatasetConfig } from "./LegacySettingsWrapper";

export const LLMSettings = (props: {
  serverConfig: Accessor<DatasetConfig>;
  setServerConfig: (config: (prev: DatasetConfig) => DatasetConfig) => void;
  saveConfig?: () => void;
}) => {
  createEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "s") {
        event.preventDefault();
        props.saveConfig?.();
      }
    };
    window.addEventListener("keydown", handleKeyDown);

    onCleanup(() => {
      window.removeEventListener("keydown", handleKeyDown);
    });
  });

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

            <div class="mt-4 grid grid-cols-4 gap-x-3 gap-y-6">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="llmAPIURL"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  LLM API URL
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Select the API URL to use for the LLM. Contact us or use the API if you need a custom URL."
                  />
                </label>
                <select
                  name="llmAPIURL"
                  id="llmAPIURL"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-2 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  value={props.serverConfig().LLM_BASE_URL?.toString()}
                  onInput={(e) =>
                    props.setServerConfig((prev) => {
                      const updatedConfig = {
                        ...prev,
                        LLM_BASE_URL: e.currentTarget.value,
                      };
                      if (prev.LLM_BASE_URL !== e.currentTarget.value) {
                        updatedConfig.LLM_API_KEY = null;
                      }

                      return updatedConfig;
                    })
                  }
                >
                  <option value="https://api.openai.com/v1">
                    https://api.openai.com/v1
                  </option>
                  <option value="https://openrouter.ai/api/v1">
                    https://openrouter.ai/api/v1
                  </option>
                  <option value="https://api.groq.com/openai/v1">
                    https://api.groq.com/openai/v1
                  </option>
                </select>
              </div>

              <div class="col-span-4 sm:col-span-2">
                <label
                  for="llmAPIURL"
                  class="flex items-center gap-2 pr-2 text-sm font-medium leading-6"
                >
                  LLM API Key
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="LLM API Key cannot be viewed after you set it"
                  />
                </label>
                <div class="flex items-center gap-2">
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
                  <button
                    type="button"
                    class="rounded-md bg-magenta-400 px-3 py-2 text-sm text-white"
                    onClick={() => {
                      props.setServerConfig((prev) => {
                        return {
                          ...prev,
                          LLM_API_KEY: "",
                        };
                      });
                      if (props.saveConfig) {
                        props.saveConfig();
                      }
                    }}
                  >
                    Reset
                  </button>
                </div>
              </div>

              <div class="col-span-4 sm:col-span-2">
                <label
                  for="llmAPIURL"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  LLM Default Model
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Select the default model to use for the LLM. See https://openrouter.ai/models for all available LLMs you can use."
                  />
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

          {/* RAG Settings */}
          <div class="mt-6">
            <span>
              <h2 class="flex items-center gap-2 text-lg font-semibold leading-6">
                RAG Settings
                <Tooltip
                  body={<AiOutlineInfoCircle />}
                  tooltipText="Control the prompt which pairs with the user query and retrieved chunks to generate the final completion."
                />
              </h2>
              <hr class="mt-2" />
            </span>
            <div class="mt-4 grid grid-cols-4 gap-x-3 gap-y-6">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="messageToQueryPrompt"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  System Prompt
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Guides the system towards a higher-level object or goal."
                  />
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
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  RAG Prompt
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="RAG prompt should focus on how to handle the retrieved context in combination with the user query to achieve the overall goal."
                  />
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

              <div class="col-span-4 sm:col-span-2">
                <label
                  for="stopTokens"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Stop Tokens
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Stop tokens are used to tell the LLM when to stop generating the query. Set to common stop tokens in your chunks or leave default if you don't have any."
                  />
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
                  for="temperature"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Temperature
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="The temperature controls the randomness of the generated completions."
                  />
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
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Presence Penalty
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="The presence penalty penalizes the model for repeating the same information."
                  />
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
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Frequency Penalty
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="The frequency penalty penalizes the model for using the same token multiple times."
                  />
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
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Max Tokens
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Maximum number of tokens to generate in the completion."
                  />
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
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="nRetrievalsToInclude"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Search Page Size
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Number of retrieved chunks to include for RAG."
                  />
                </label>
                <input
                  name="nRetrievalsToInclude"
                  type="number"
                  placeholder="Number of retrievals to include for RAG"
                  id="linesBeforeShowMore"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  value={props
                    .serverConfig()
                    .N_RETRIEVALS_TO_INCLUDE?.toString()}
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
            </div>
          </div>

          {/* HyDE Settings */}
          <div class="mt-6">
            <span>
              <h2 class="flex items-center gap-2 text-lg font-semibold leading-6">
                HyDE (Hypothetical Document Embeddings) Settings
                <Tooltip
                  body={<AiOutlineInfoCircle />}
                  tooltipText="HyDE settings are used to configure the prompt which is used to transform user prompts into search queries which hit the retrieval sub-system."
                />
              </h2>
              <hr class="mt-2" />
            </span>

            <div class="mt-4 grid grid-cols-4 gap-x-6 gap-y-3">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="messageToQueryPrompt"
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Message to Query Prompt
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Message to Query prompt is used to tell the LLM how to convert the user message into a search query."
                  />
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

              <div class="col-span-4 flex items-center space-x-2">
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
                  class="flex items-center gap-2 text-sm font-medium leading-6"
                >
                  Use Message to Query Prompt (HyDE)
                  <Tooltip
                    body={<AiOutlineInfoCircle />}
                    tooltipText="Must be checked in order the HyDE system to be used during RAG."
                  />
                </label>
              </div>
            </div>
          </div>
        </div>
      </div>
    </form>
  );
};
