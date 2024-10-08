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
import { CrawlInterval, CrawlOptions, DistanceMetric } from "trieve-ts-sdk";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { Tooltip } from "shared/ui";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";
import { createStore, SetStoreFunction, unwrap } from "solid-js/store";
import { DatasetConfig } from "./dataset-settings/LegacySettingsWrapper";
import { cn } from "shared/utils";

export interface NewDatasetModalProps {
  isOpen: Accessor<boolean>;
  closeModal: () => void;
}

export const NewDatasetModal = (props: NewDatasetModalProps) => {
  const userContext = useContext(UserContext);
  const navigate = useNavigate();

  const [serverConfig, setServerConfig] = createStore(
    defaultServerEnvsConfiguration,
  );
  const [crawlOptions, setCrawlOptions] = createSignal<CrawlOptions>();
  const [name, setName] = createSignal<string>("");
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [showScraping, setShowScraping] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [fillWithExampleData, setFillWithExampleData] = createSignal(false);

  const createDataset = async () => {
    const curServerConfig = unwrap(serverConfig);

    try {
      setIsLoading(true);
      const dataset = await createNewDataset({
        name: name(),
        organizationId: userContext.selectedOrg().id,
        serverConfig: curServerConfig,
        crawlOptions: crawlOptions(),
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
                            <FiChevronDown />
                          </Match>
                          <Match when={showScraping()}>
                            <FiChevronUp />
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
                        <div class="ml-4 flex flex-col space-y-2 border-neutral-900/10 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t">
                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="scrapingUrl"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              Scraping URL
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="The URL of the website you would like to scrape."
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="scrapingUrl"
                              name="scrapingUrl"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.site_url ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="URL Patterns to exclude from the crawl. Example: '/admin/*, /login/*"
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="excludePaths"
                              name="excludePaths"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.exclude_paths ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      exclude_paths:
                                        e.currentTarget.value.split(","),
                                    };
                                  }

                                  return {
                                    ...prev,
                                    exclude_paths:
                                      e.currentTarget.value.split(","),
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="URL Patterns to include in the crawl. Example: '/docs/*, /blog/*'"
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="includePaths"
                              name="includePaths"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.include_paths ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      include_paths:
                                        e.currentTarget.value.split(","),
                                    };
                                  }

                                  return {
                                    ...prev,
                                    include_paths:
                                      e.currentTarget.value.split(","),
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Specify the HTML tags, classes and ids to exclude from the response. Example 'header, .table-of-contents'"
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="excludeTags"
                              name="excludeTags"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.exclude_tags ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      exclude_tags:
                                        e.currentTarget.value.split(","),
                                    };
                                  }

                                  return {
                                    ...prev,
                                    exclude_tags:
                                      e.currentTarget.value.split(","),
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Specify the HTML tags, classes and ids to include in the response. Example 'article, .inner-content'"
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="includeTags"
                              name="includeTags"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.include_tags ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      include_tags:
                                        e.currentTarget.value.split(","),
                                    };
                                  }

                                  return {
                                    ...prev,
                                    include_tags:
                                      e.currentTarget.value.split(","),
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="How many levels deep to crawl, defaults to 10"
                                direction="right"
                              />
                            </label>
                            <input
                              type="number"
                              id="maxDepth"
                              name="maxDepth"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.max_depth ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      max_depth: parseInt(
                                        e.currentTarget.value,
                                      ),
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Maximum number of pages to crawl, defaults to 1000"
                                direction="right"
                              />
                            </label>
                            <input
                              type="number"
                              id="pageLimit"
                              name="pageLimit"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.limit ?? ""}
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="How often to scrape the website. Defaults to daily."
                                direction="right"
                              />
                            </label>
                            <select
                              id="interval"
                              name="interval"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={crawlOptions()?.interval ?? ""}
                              onChange={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      interval: e.currentTarget
                                        .value as CrawlInterval,
                                    };
                                  }

                                  return {
                                    ...prev,
                                    interval: e.currentTarget
                                      .value as CrawlInterval,
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="Boost the frequency of titles in the search index such that title matches are prioritized."
                                direction="right"
                              />
                            </label>
                            <input
                              type="checkbox"
                              id="boostTitles"
                              name="boostTitles"
                              class="col-span-2 mt-2.5 block w-fit rounded-md border-[0.5px] border-neutral-300 bg-white px-3 text-start placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              checked={crawlOptions()?.boost_titles ?? true}
                              onChange={(e) =>
                                setCrawlOptions((prev) => {
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

                          <div class="content-center py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                            <label
                              for="openapiSchemaUrl"
                              class="flex h-full items-center gap-2 pt-1.5 text-sm font-medium leading-6"
                            >
                              OpenAPI Schema URL
                              <Tooltip
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="URL that will return a *.json or *.yaml file with an OpenAPI schema which pairs with the docs."
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="openapiSchemaUrl"
                              name="openapiSchemaUrl"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={
                                crawlOptions()?.openapi_options
                                  ?.openapi_schema_url ?? ""
                              }
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      openapi_options: {
                                        openapi_schema_url:
                                          e.currentTarget.value,
                                        openapi_tag: "",
                                      },
                                    };
                                  }

                                  return {
                                    ...prev,
                                    openapi_options: {
                                      openapi_schema_url: e.currentTarget.value,
                                      openapi_tag:
                                        prev.openapi_options?.openapi_tag ?? "",
                                    },
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
                                body={
                                  <FaRegularCircleQuestion class="h-4 w-4 text-black" />
                                }
                                tooltipText="For a site like https://docs.trieve.ai, the tag here would be 'api-reference' because of the API routes being documented at https://docs.trieve.ai/api-reference/* paths."
                                direction="right"
                              />
                            </label>
                            <input
                              type="text"
                              id="openapiTag"
                              name="openapiTag"
                              class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                              value={
                                crawlOptions()?.openapi_options?.openapi_tag ??
                                ""
                              }
                              onInput={(e) =>
                                setCrawlOptions((prev) => {
                                  if (!prev) {
                                    return {
                                      openapi_options: {
                                        openapi_schema_url: "",
                                        openapi_tag: e.currentTarget.value,
                                      },
                                    };
                                  }

                                  return {
                                    ...prev,
                                    openapi_options: {
                                      openapi_schema_url:
                                        prev.openapi_options
                                          ?.openapi_schema_url ?? "",
                                      openapi_tag: e.currentTarget.value,
                                    },
                                  };
                                })
                              }
                            />
                          </div>
                        </div>
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

const BM25Settings = (props: {
  config: DatasetConfig;
  setConfig: SetStoreFunction<DatasetConfig>;
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
            step={0.1}
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_B || "0.75"}
            onInput={(e) => {
              props.setConfig("BM25_B", parseFloat(e.currentTarget.value));
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
        </div>
        <div>
          <label class="block">K</label>
          <input
            min={0}
            step={0.25}
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_K || "1.2"}
            onInput={(e) => {
              props.setConfig("BM25_K", parseFloat(e.currentTarget.value));
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
        </div>
        <div>
          <label class="block">Average Length</label>
          <input
            step={1}
            disabled={!props.config.BM25_ENABLED}
            value={props.config.BM25_AVG_LEN || "256"}
            onInput={(e) => {
              props.setConfig(
                "BM25_AVG_LEN",
                parseFloat(e.currentTarget.value),
              );
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 group-disabled:opacity-20 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
        </div>
      </div>
    </div>
  );
};
