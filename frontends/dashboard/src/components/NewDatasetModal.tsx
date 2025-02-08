import {
  Accessor,
  createSignal,
  useContext,
  For,
  Switch,
  Match,
  Show,
  createEffect,
  onCleanup,
} from "solid-js";
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
  availableDistanceMetrics,
  availableEmbeddingModels,
  availableRerankerModels,
} from "shared/types";
import { createToast } from "./ShowToasts";
import { createNewDataset } from "../api/createDataset";
import { uploadSampleData } from "../api/uploadSampleData";
import { defaultServerEnvsConfiguration } from "../utils/serverEnvs";
import { DistanceMetric } from "trieve-ts-sdk";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { Tooltip } from "shared/ui";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { createStore, SetStoreFunction, unwrap } from "solid-js/store";
import { DatasetConfig } from "./dataset-settings/LegacySettingsWrapper";
import { cn } from "shared/utils";
import { ValidateFn, ErrorMsg, ValidateErrors } from "../utils/validation";

export interface NewDatasetModalProps {
  isOpen: Accessor<boolean>;
  closeModal: () => void;
}

const validate: ValidateFn<DatasetConfig> = (value) => {
  const errors: ValidateErrors<DatasetConfig> = {};

  if (value.BM25_ENABLED) {
    if (!value.BM25_B) {
      errors.BM25_B = "B is required";
    } else if (value.BM25_B < 0) {
      errors.BM25_B = "B must be greater than 0";
    } else if (value.BM25_B > 1) {
      errors.BM25_B = "B must be less than 1";
    }
    if (!value.BM25_K) {
      errors.BM25_K = "K is required";
    } else if (value.BM25_K < 0) {
      errors.BM25_K = "K must be greater than 0";
    }
    if (!value.BM25_AVG_LEN) {
      errors.BM25_AVG_LEN = "Average Length is required";
    }
  }

  return {
    errors,
    valid: Object.values(errors).filter((v) => !!v).length === 0,
  };
};

export const NewDatasetModal = (props: NewDatasetModalProps) => {
  const userContext = useContext(UserContext);
  const navigate = useNavigate();

  const [serverConfig, setServerConfig] = createStore(
    defaultServerEnvsConfiguration,
  );
  const [name, setName] = createSignal<string>("");
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [fillWithExampleData, setFillWithExampleData] = createSignal(false);

  const [errors, setErrors] = createStore<
    ReturnType<ValidateFn<DatasetConfig>>["errors"]
  >({});

  const createDataset = async () => {
    const curServerConfig = unwrap(serverConfig);
    const validateResult = validate(curServerConfig);
    if (validateResult.valid) {
      setErrors({});
    } else {
      console.log(validateResult.errors);
      setErrors(validateResult.errors);
      return;
    }
    try {
      setIsLoading(true);
      const dataset = await createNewDataset({
        name: name(),
        organizationId: userContext.selectedOrg().id,
        serverConfig: curServerConfig,
      });

      if (fillWithExampleData()) {
        await uploadSampleData({
          datasetId: dataset.id,
        });
      }

      createToast({
        title: "Success",
        type: "success",
        message: "Successfully created dataset",
      });

      props.closeModal();

      setIsLoading(false);
      await userContext.login();
      await new Promise((resolve) => setTimeout(resolve, 500));
      navigate(`/dataset/${dataset.id}`);
    } catch (e: unknown) {
      setIsLoading(false);
      const error = e as Error;
      createToast({
        title: "Error",
        type: "error",
        message: error.message,
      });
    }
  };

  createEffect(() => {
    if (props.isOpen()) {
      const keydownListener = (e: KeyboardEvent) => {
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          void createDataset();
        }
      };
      window.addEventListener("keydown", keydownListener);
      onCleanup(() => {
        window.removeEventListener("keydown", keydownListener);
      });
    }
  });

  return (
    <Transition appear show={props.isOpen()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-20 overflow-y-auto"
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
            <DialogPanel class="inline-block max-h-[90vh] w-full max-w-2xl transform overflow-hidden overflow-y-auto rounded-md bg-white p-6 text-left align-middle shadow-xl transition-all">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  void createDataset();
                }}
              >
                <div class="space-y-12 sm:space-y-16">
                  <div>
                    <DialogTitle
                      as="h3"
                      class="text-base font-semibold leading-7"
                    >
                      Create New Dataset
                    </DialogTitle>

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
                            <option>{userContext.selectedOrg().name}</option>
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
                              {userContext.selectedOrg().name}/
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

                      <div>
                        <div class="py-4 sm:grid sm:grid-cols-3 sm:items-baseline sm:gap-4">
                          <label
                            for="fill-with-example-data"
                            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                          >
                            Fill with Example Data
                            <Tooltip
                              body={
                                <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                              }
                              tooltipText="If selected, we will pre-fill the dataset with a random selection of Y-Combinator companies so you can immediately test the product."
                            />
                          </label>
                          <div class="mt-4 sm:col-span-2 sm:mt-0">
                            <input
                              type="checkbox"
                              name="fill-with-example-data"
                              id="fill-with-example-data"
                              class="rounded-md border border-neutral-300 bg-white py-1.5 pl-2 pr-10 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              checked={fillWithExampleData()}
                              onChange={(e) =>
                                setFillWithExampleData(e.currentTarget.checked)
                              }
                            />
                          </div>
                        </div>
                      </div>

                      <button
                        class="flex w-full flex-row items-center gap-2 py-4 text-sm font-medium"
                        onClick={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          setShowAdvanced(!showAdvanced());
                        }}
                      >
                        <Switch>
                          <Match when={!showAdvanced()}>
                            <FiChevronDown />
                          </Match>
                          <Match when={showAdvanced()}>
                            <FiChevronUp />
                          </Match>
                        </Switch>
                        Configuration
                        <Tooltip
                          body={
                            <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                          }
                          tooltipText="Change your default embedding model and distance metric."
                        />
                      </button>
                      <Show when={showAdvanced()}>
                        <div class="ml-4 flex flex-col space-y-2 border-neutral-900/10 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t">
                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="embeddingSize"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Dense Vector Embedding Model{" "}
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                                }
                                tooltipText="Dense vector models are used for semantic search. jina-base-en provides the best balance of latency and relevance quality. Only change this if you have a specific requirement. Custom models are supported on the enterprise plan."
                              />
                            </label>
                            <select
                              id="embeddingSize"
                              name="embeddingSize"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={
                                availableEmbeddingModels.find(
                                  (model) =>
                                    model.id ===
                                    serverConfig.EMBEDDING_MODEL_NAME,
                                )?.name ?? availableEmbeddingModels[0].name
                              }
                              onChange={(e) => {
                                const selectedModel =
                                  availableEmbeddingModels.find(
                                    (model) =>
                                      model.name === e.currentTarget.value,
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
                                  <option value={model.name}>
                                    {model.name}
                                  </option>
                                )}
                              </For>
                            </select>
                          </div>

                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="embeddingSize"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Reranker Model{" "}
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                                }
                                tooltipText="Reranker Model for re-ranking search results."
                              />
                            </label>
                            <select
                              id="embeddingSize"
                              name="embeddingSize"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={
                                availableRerankerModels.find(
                                  (model) =>
                                    model.id ===
                                    serverConfig.RERANKER_MODEL_NAME,
                                )?.name ?? availableRerankerModels[0].name
                              }
                              onChange={(e) => {
                                const selectedModel =
                                  availableRerankerModels.find(
                                    (model) =>
                                      model.name === e.currentTarget.value,
                                  );

                                setServerConfig((prev) => {
                                  return {
                                    ...prev,
                                    RERANKER_MODEL_NAME: selectedModel?.id,
                                  };
                                });
                              }}
                            >
                              <For each={availableRerankerModels}>
                                {(model) => (
                                  <option value={model.name}>
                                    {model.name}
                                  </option>
                                )}
                              </For>
                            </select>
                          </div>
                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="distanceMetric"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Distance Metric
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                                }
                                tooltipText="Cosine will almost always be best. Only change if you are confident that your data is unique and requires a different metric."
                              />
                            </label>
                            <select
                              id="distanceMetric"
                              name="distanceMetric"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={
                                availableDistanceMetrics.find(
                                  (model) =>
                                    model.id === serverConfig.DISTANCE_METRIC,
                                )?.name ?? availableDistanceMetrics[0].name
                              }
                              onChange={(e) => {
                                const distanceMetric =
                                  availableDistanceMetrics.find(
                                    (metric) =>
                                      metric.name === e.currentTarget.value,
                                  );

                                // @ts-expect-error circular type import fix later
                                setServerConfig((prev) => {
                                  return {
                                    ...prev,
                                    DISTANCE_METRIC:
                                      distanceMetric?.id ??
                                      ("cosine" as DistanceMetric),
                                  };
                                });
                              }}
                            >
                              <For each={availableDistanceMetrics}>
                                {(metric) => (
                                  <option value={metric.name}>
                                    {metric.name}
                                  </option>
                                )}
                              </For>
                            </select>
                          </div>

                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="sparseVector"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Sparse Vector Model{" "}
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                                }
                                tooltipText="Sparse vector models are used for fulltext search. In contrast to keyword bm25, fulltext is aware of the most important terms in your query. Currently only splade-v3 is offered."
                              />
                            </label>
                            <p
                              id="sparseVector"
                              class="col-span-2 block w-full bg-white px-3 py-1.5 text-sm text-neutral-700"
                            >
                              Dataset will be configured with{" "}
                              <a
                                href="https://huggingface.co/naver/splade-v3"
                                target="_blank"
                                class="underline"
                              >
                                splade-v3
                              </a>
                              . Other sparse encoders, including custom models,
                              are supported on the enterprise plan.
                            </p>
                          </div>

                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="bm25"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              BM25{" "}
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                                }
                                tooltipText="BM25 is used for keyword search. It is enabled on all datasets by default."
                              />
                            </label>
                            <p
                              id="bm25"
                              class="col-span-2 block w-full bg-white px-3 py-1.5 text-sm text-neutral-700"
                            >
                              <BM25Settings
                                errors={errors}
                                config={serverConfig}
                                setConfig={setServerConfig}
                              />
                            </p>
                          </div>
                        </div>
                      </Show>

                      <div class="flex w-full flex-row items-center gap-2 py-4 text-sm">
                        <div class="rounded-md bg-blue-50 p-4">
                          <div class="flex">
                            <div class="flex-shrink-0">
                              <svg
                                class="h-5 w-5 text-blue-400"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                                aria-hidden="true"
                                data-slot="icon"
                              >
                                <path
                                  fill-rule="evenodd"
                                  d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-7-4a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM9 9a.75.75 0 0 0 0 1.5h.253a.25.25 0 0 1 .244.304l-.459 2.066A1.75 1.75 0 0 0 10.747 15H11a.75.75 0 0 0 0-1.5h-.253a.25.25 0 0 1-.244-.304l.459-2.066A1.75 1.75 0 0 0 9.253 9H9Z"
                                  clip-rule="evenodd"
                                />
                              </svg>
                            </div>
                            <div class="ml-3 flex-1 md:flex md:justify-between">
                              <p class="text-sm text-blue-700">
                                Scraping settings for websites, forums, shopify
                                stores, and more can be setup in Crawling
                                Settings tab after dataset creation.
                              </p>
                            </div>
                          </div>
                        </div>
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
                    disabled={name() === "" || isLoading()}
                    class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm focus:outline-magenta-700 disabled:bg-magenta-200"
                  >
                    Create New Dataset
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

const BM25Settings = (props: {
  config: DatasetConfig;
  setConfig: SetStoreFunction<DatasetConfig>;
  errors: ReturnType<ValidateFn<DatasetConfig>>["errors"];
}) => {
  return (
    <div>
      <div class="flex items-center gap-2 py-2 pb-5">
        <label class="block">BM25 Enabled</label>
        <input
          checked={props.config.BM25_ENABLED ? true : false}
          onChange={(e) => {
            props.setConfig("BM25_ENABLED", e.currentTarget.checked);
          }}
          class="h-3 w-3 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
      </div>
      <div
        class={cn(
          "group flex justify-stretch gap-2",
          !props.config.BM25_ENABLED && "opacity-40",
        )}
      >
        <div>
          <label class="block">B</label>
          <input
            min={0}
            step="any"
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_B?.toString() || ""}
            onInput={(e) => {
              props.setConfig("BM25_B", parseFloat(e.currentTarget.value));
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
          <ErrorMsg error={props.errors.BM25_B} />
        </div>
        <div>
          <label class="block">K</label>
          <input
            step="any"
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_K?.toString() || ""}
            onInput={(e) => {
              props.setConfig("BM25_K", parseFloat(e.currentTarget.value));
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
          <ErrorMsg error={props.errors.BM25_K} />
        </div>
        <div>
          <label class="block">Average Length</label>
          <input
            step={1}
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_AVG_LEN?.toString() || ""}
            onInput={(e) => {
              props.setConfig(
                "BM25_AVG_LEN",
                parseFloat(e.currentTarget.value),
              );
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
          <ErrorMsg error={props.errors.BM25_AVG_LEN} />
        </div>
      </div>
    </div>
  );
};

export default NewDatasetModal;
