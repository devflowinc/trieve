import {
  Accessor,
  createSignal,
  useContext,
  For,
  Switch,
  Match,
  Show,
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
} from "shared/types";
import { createToast } from "./ShowToasts";
import { createNewDataset } from "../api/createDataset";
import { uploadSampleData } from "../api/uploadSampleData";
import { defaultServerEnvsConfiguration } from "../utils/serverEnvs";
import { CrawlInterval, DistanceMetric } from "trieve-ts-sdk";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { Tooltip } from "shared/ui";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { createStore, SetStoreFunction, unwrap } from "solid-js/store";
import { DatasetConfig } from "./dataset-settings/LegacySettingsWrapper";
import { cn } from "shared/utils";
import { ValidateFn, ErrorMsg, ValidateErrors } from "../utils/validation";
import {
  defaultCrawlOptions,
  FlatCrawlOptions,
  flattenCrawlOptions,
  unflattenCrawlOptions,
  validateFlatCrawlOptions,
} from "../pages/dataset/CrawlingSettings";

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
  const [crawlOptions, setCrawlOptions] = createStore<FlatCrawlOptions>(
    flattenCrawlOptions(defaultCrawlOptions),
  );
  const [name, setName] = createSignal<string>("");
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [showScraping, setShowScraping] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [fillWithExampleData, setFillWithExampleData] = createSignal(false);

  const [errors, setErrors] = createStore<
    ReturnType<ValidateFn<DatasetConfig>>["errors"]
  >({});
  const [crawlErrors, setCrawlErrors] = createStore<
    ReturnType<ValidateFn<FlatCrawlOptions>>["errors"]
  >({});

  const createDataset = async () => {
    const curServerConfig = unwrap(serverConfig);
    const unwrappedFlatCrawlOptions = unwrap(crawlOptions);
    const validateResult = validate(curServerConfig);
    if (validateResult.valid) {
      setErrors({});
    } else {
      setErrors(validateResult.errors);
      return;
    }

    if (showScraping()) {
      const crawlValidateResult = validateFlatCrawlOptions(
        unwrappedFlatCrawlOptions,
      );
      if (crawlValidateResult.valid) {
        setCrawlErrors({});
      } else {
        setCrawlErrors(crawlValidateResult.errors);
        return;
      }
    }

    try {
      setIsLoading(true);
      const dataset = await createNewDataset({
        name: name(),
        organizationId: userContext.selectedOrg().id,
        serverConfig: curServerConfig,
        crawlOptions: showScraping()
          ? unflattenCrawlOptions(unwrappedFlatCrawlOptions)
          : undefined,
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
                                <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                              }
                              tooltipText="If selected, we will pre-fill the dataset with a random selection of Y-Combinator companies so you can immediately test the product."
                              direction="right"
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
                            <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                          }
                          tooltipText="Change your default embedding model and distance metric."
                          direction="right"
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
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Dense vector models are used for semantic search. jina-base-en provides the best balance of latency and relevance quality. Only change this if you have a specific requirement. Custom models are supported on the enterprise plan."
                                direction="right"
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
                              for="distanceMetric"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Distance Metric
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Cosine will almost always be best. Only change if you are confident that your data is unique and requires a different metric."
                                direction="right"
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
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Sparse vector models are used for fulltext search. In contrast to keyword bm25, fulltext is aware of the most important terms in your query. Currently only splade-v3 is offered."
                                direction="right"
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
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="BM25 is used for keyword search. It is enabled on all datasets by default."
                                direction="right"
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

                      <button
                        class="flex w-full flex-row items-center gap-2 py-4 text-sm font-medium"
                        onClick={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          setShowScraping((prev) => !prev);
                        }}
                      >
                        <Switch>
                          <Match when={!showScraping()}>
                            <input
                              class="h-3 w-3 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
                              type="checkbox"
                              checked={false}
                            />
                          </Match>
                          <Match when={showScraping()}>
                            <input
                              class="h-3 w-3 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
                              type="checkbox"
                              checked={true}
                            />
                          </Match>
                        </Switch>
                        Scraping
                        <Tooltip
                          body={
                            <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                          }
                          tooltipText="Configure your dataset to be populated by scraping a particular website."
                          direction="right"
                        />
                      </button>
                      <Show when={showScraping()}>
                        <ScrapingSettings
                          crawlOptions={crawlOptions}
                          setCrawlOptions={setCrawlOptions}
                          errors={crawlErrors}
                        />
                      </Show>
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
export default NewDatasetModal;

const ScrapingSettings = (props: {
  crawlOptions: FlatCrawlOptions;
  setCrawlOptions: SetStoreFunction<FlatCrawlOptions>;
  errors: ReturnType<ValidateFn<FlatCrawlOptions>>["errors"];
}) => {
  return (
    <div>
      <div class="ml-4 flex flex-col space-y-2 border-neutral-900/10 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t">
        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="scrapingUrl"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Scraping URL
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="The URL of the website you would like to scrape."
              direction="right"
            />
          </label>
          <input
            type="text"
            id="scrapingUrl"
            name="scrapingUrl"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.site_url ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    site_url: e.currentTarget.value,
                  };
                }

                return {
                  ...prev,
                  site_url: e.currentTarget.value,
                };
              })
            }
          />
        </div>
        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="excludePaths"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Exclude Paths
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="URL Patterns to exclude from the crawl. Example: '/admin/*, /login/*"
              direction="right"
            />
          </label>
          <input
            type="text"
            id="excludePaths"
            name="excludePaths"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.exclude_paths ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    exclude_paths: e.currentTarget.value.split(","),
                  };
                }

                return {
                  ...prev,
                  exclude_paths: e.currentTarget.value.split(","),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="includePaths"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Include Paths
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="URL Patterns to include in the crawl. Example: '/docs/*, /blog/*'"
              direction="right"
            />
          </label>
          <input
            type="text"
            id="includePaths"
            name="includePaths"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.include_paths ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    include_paths: e.currentTarget.value.split(","),
                  };
                }

                return {
                  ...prev,
                  include_paths: e.currentTarget.value.split(","),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="excludeTags"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Exclude Tags
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="Specify the HTML tags, classes and ids to exclude from the response. Example 'header, .table-of-contents'"
              direction="right"
            />
          </label>
          <input
            type="text"
            id="excludeTags"
            name="excludeTags"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.exclude_tags ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    exclude_tags: e.currentTarget.value.split(","),
                  };
                }
                return {
                  ...prev,
                  exclude_tags: e.currentTarget.value.split(","),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="includeTags"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Include Tags
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="Specify the HTML tags, classes and ids to include in the response. Example 'article, .inner-content'"
              direction="right"
            />
          </label>
          <input
            type="text"
            id="includeTags"
            name="includeTags"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.include_tags ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    include_tags: e.currentTarget.value.split(","),
                  };
                }

                return {
                  ...prev,
                  include_tags: e.currentTarget.value.split(","),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="maxDepth"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Max Depth
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="How many levels deep to crawl, defaults to 10"
              direction="right"
            />
          </label>
          <input
            type="number"
            id="maxDepth"
            name="maxDepth"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.max_depth ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    max_depth: parseInt(e.currentTarget.value),
                  };
                }

                return {
                  ...prev,
                  max_depth: parseInt(e.currentTarget.value),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="pageLimit"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Limit (Max Pages to Crawl)
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="Maximum number of pages to crawl, defaults to 1000"
              direction="right"
            />
          </label>
          <input
            type="number"
            id="pageLimit"
            name="pageLimit"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.limit ?? ""}
            onInput={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    limit: parseInt(e.currentTarget.value),
                  };
                }

                return {
                  ...prev,
                  limit: parseInt(e.currentTarget.value),
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="interval"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Interval
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="How often to scrape the website. Defaults to daily."
              direction="right"
            />
          </label>
          <select
            id="interval"
            name="interval"
            class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            value={props.crawlOptions?.interval ?? ""}
            onChange={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    interval: e.currentTarget.value as CrawlInterval,
                  };
                }

                return {
                  ...prev,
                  interval: e.currentTarget.value as CrawlInterval,
                };
              })
            }
          >
            <option value="daily">daily</option>
            <option value="weekly">weekly</option>
            <option value="monthly">monthly</option>
          </select>
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
          <label
            for="boostTitles"
            class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
          >
            Boost Titles
            <Tooltip
              body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
              tooltipText="Boost the frequency of titles in the search index such that title matches are prioritized."
              direction="right"
            />
          </label>
          <input
            type="checkbox"
            id="boostTitles"
            name="boostTitles"
            class="col-span-2 mt-2.5 block w-fit rounded-md border-[0.5px] border-neutral-300 bg-white px-3 text-start placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            checked={props.crawlOptions?.boost_titles ?? true}
            onChange={(e) =>
              props.setCrawlOptions((prev) => {
                if (!prev) {
                  return {
                    boost_titles: e.currentTarget.checked,
                  };
                }

                return {
                  ...prev,
                  boost_titles: e.currentTarget.checked,
                };
              })
            }
          />
        </div>

        <div class="content-center py-4 sm:grid sm:grid-cols-2 sm:items-start sm:gap-4">
          <div class="col-span-1 flex">
            <input
              type="checkbox"
              id="useOpenAPI"
              name="useOpenAPI"
              class="mt-2.5 block w-fit rounded-md border-[0.5px] border-neutral-300 bg-white px-3 text-start placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              checked={props.crawlOptions.type === "openapi"}
              onChange={(e) =>
                props.setCrawlOptions((prev) => {
                  if (!e.currentTarget.checked) {
                    if (prev.type === "openapi") {
                      return {
                        ...prev,
                        type: undefined,
                      };
                    }
                    return {
                      ...prev,
                    };
                  } else {
                    return {
                      ...prev,
                      type: "openapi",
                    };
                  }
                })
              }
            />

            <label
              for="useOpenAPI"
              class="flex h-full items-center gap-2 pl-1.5 pt-1.5 text-sm font-medium leading-6"
            >
              OpenAPI spec
              <Tooltip
                body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
                tooltipText="Parse an OpenAPI spec on a specified route"
                direction="right"
              />
            </label>
          </div>
          <div class="col-span-1 flex">
            <input
              type="checkbox"
              id="useShopify"
              name="useShopify"
              class="mt-2.5 block w-fit rounded-md border-[0.5px] border-neutral-300 bg-white px-3 text-start placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              checked={props.crawlOptions.type == "shopify"}
              onChange={(e) =>
                props.setCrawlOptions((prev) => {
                  if (!e.currentTarget.checked) {
                    if (prev.type === "shopify") {
                      return {
                        ...prev,
                        type: undefined,
                      };
                    }
                    return {
                      ...prev,
                    };
                  } else {
                    return {
                      type: "shopify" as const,
                    };
                  }
                })
              }
            />

            <label
              for="useShopify"
              class="flex h-full items-center gap-2 pl-1.5 pt-1.5 text-sm font-medium leading-6"
            >
              Is Shopify Store
              <Tooltip
                body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
                tooltipText="Toggle if the webpage is a shopify store to scrape the products more accurately"
                direction="left"
              />
            </label>
          </div>
        </div>

        <Switch>
          <Match when={props.crawlOptions.type == "openapi"}>
            <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
              <label
                for="openapiSchemaUrl"
                class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
              >
                OpenAPI Schema URL
                <Tooltip
                  body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
                  tooltipText="URL that will return a *.json or *.yaml file with an OpenAPI schema which pairs with the docs."
                  direction="right"
                />
              </label>
              <input
                type="text"
                id="openapiSchemaUrl"
                name="openapiSchemaUrl"
                class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={props.crawlOptions.openapi_schema_url ?? ""}
                onInput={(e) =>
                  props.setCrawlOptions((prev) => {
                    return {
                      ...prev,
                      type: "openapi",
                      openapi_schema_url: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>

            <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
              <label
                for="openapiTag"
                class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
              >
                OpenAPI Tag
                <Tooltip
                  body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
                  tooltipText="For a site like https://docs.trieve.ai, the tag here would be 'api-reference' because of the API routes being documented at https://docs.trieve.ai/api-reference/* paths."
                  direction="right"
                />
              </label>
              <input
                type="text"
                id="openapiTag"
                name="openapiTag"
                class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                value={props.crawlOptions.openapi_tag ?? ""}
                onInput={(e) =>
                  props.setCrawlOptions((prev) => {
                    return {
                      ...prev,
                      type: "openapi",
                      openapi_tag: e.currentTarget.value,
                    };
                  })
                }
              />
            </div>
          </Match>
          <Match when={props.crawlOptions.type == "shopify"}>
            <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
              <label
                for="groupVariants"
                class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
              >
                Group Product Variants
                <Tooltip
                  body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
                  tooltipText="This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product"
                  direction="right"
                />
              </label>
              <input
                type="checkbox"
                id="groupVariants"
                name="groupVariants"
                class="col-span-2 mt-2.5 block w-fit rounded-md border-[0.5px] border-neutral-300 bg-white px-3 text-start placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                checked={props.crawlOptions.group_variants ?? true}
                onChange={(e) =>
                  props.setCrawlOptions((prev) => {
                    return {
                      ...prev,
                      scrape_options: {
                        type: "shopify",
                        group_variants: e.currentTarget.checked,
                      },
                    };
                  })
                }
              />
            </div>
          </Match>
        </Switch>
      </div>
    </div>
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
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
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
