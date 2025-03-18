import { createQuery } from "@tanstack/solid-query";
import { FaSolidCheck } from "solid-icons/fa";
import { createMemo, Show, useContext } from "solid-js";
import { useTrieve } from "../hooks/useTrieve";
import { UserContext } from "../contexts/UserContext";

export const OnboardingSteps = () => {
  const userContext = useContext(UserContext);
  const usageQuery = createQuery(() => ({
    queryKey: ["org-usage", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch("/api/organization/usage/{organization_id}", "post", {
        organizationId: userContext.selectedOrg().id,
        data: {},
      });
    },
  }));
  const trieve = useTrieve();
  const searchAnalyticsQuery = createQuery(() => ({
    queryKey: ["top-datasets", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch("/api/analytics/top", "post", {
        organizationId: userContext.selectedOrg().id,
        data: {
          type: "search",
        },
      });
    },
  }));

  const ragAnalyticsQuery = createQuery(() => ({
    queryKey: ["top-datasets", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch("/api/analytics/top", "post", {
        organizationId: userContext.selectedOrg().id,
        data: {
          type: "rag",
        },
      });
    },
  }));

  const activeDataset = createMemo(() => {
    if (!usageQuery.isSuccess) {
      return 999;
    }
    if ((usageQuery.data?.dataset_count ?? 0) == 0) {
      return 1;
    } else if ((usageQuery.data?.chunk_count ?? 0) == 0) {
      return 2;
    } else if (
      searchAnalyticsQuery.data?.find(
        (top_dataset) => top_dataset.total_queries > 0,
      ) &&
      ragAnalyticsQuery.data?.find(
        (top_dataset) => top_dataset.total_queries > 0,
      )
    ) {
      return 4;
    }

    return 3;
  });

  return (
    <Show when={activeDataset() < 4}>
      <div class="rounded-lg bg-white shadow lg:border-b lg:border-t lg:border-gray-200">
        <nav class="w-full" aria-label="Progress">
          <ol
            role="list"
            class="w-full overflow-hidden rounded-md lg:flex lg:rounded-none lg:border-l lg:border-r lg:border-gray-200"
          >
            <li class="relative w-full overflow-hidden">
              <div class="w-full overflow-hidden rounded-t-md border border-b-0 border-gray-200 lg:border-0">
                {/* <!-- Completed Step --> */}
                <div class="group">
                  <span
                    classList={{
                      "absolute left-0 top-0 h-full w-1 lg:bottom-0 lg:top-auto lg:h-0.5 lg:w-full":
                        true,
                      "bg-fuchsia-400": activeDataset() == 1,
                      "bg-transparent": activeDataset() != 1,
                    }}
                    aria-hidden="true"
                  />
                  <span class="flex items-start px-6 py-5 text-sm">
                    <span class="flex-shrink-0">
                      <Show
                        when={activeDataset() != 1}
                        fallback={
                          <span class="flex h-10 w-10 items-center justify-center rounded-full border-2 border-fuchsia-400">
                            <span class="font-medium text-fuchsia-600">01</span>
                          </span>
                        }
                      >
                        <span class="flex h-10 w-10 items-center justify-center rounded-full bg-fuchsia-600">
                          <FaSolidCheck
                            class="h-6 w-6 fill-current text-white"
                            aria-hidden="true"
                          />
                        </span>
                      </Show>
                    </span>
                    <span class="ml-4 mt-0.5 flex min-w-0 flex-col">
                      <span
                        classList={{
                          "text-sm font-medium": true,
                          "text-fuchsia-600": activeDataset() == 1,
                        }}
                      >
                        Create a Dataset
                      </span>
                      <span class="text-sm">
                        Trieve's top-level storage container is a Dataset. Each
                        Dataset is encrypted and isolated from other Datasets.
                        API keys and requests are scoped to a specific Dataset.
                        Click on the{" "}
                        <span class="font-medium">Create Dataset</span> button
                        below to get started.
                      </span>
                    </span>
                  </span>
                </div>
              </div>
            </li>
            <li class="relative w-full overflow-hidden">
              <div class="w-full overflow-hidden border border-gray-200 lg:border-0">
                {/* <!-- Current Step --> */}
                <div>
                  <span
                    classList={{
                      "absolute left-0 top-0 h-full w-1 lg:bottom-0 lg:top-auto lg:h-0.5 lg:w-full":
                        true,
                      "bg-fuchsia-600": activeDataset() == 2,
                      "bg-transparent": activeDataset() != 2,
                    }}
                    aria-hidden="true"
                  />
                  <span class="flex items-start px-6 py-5 text-sm lg:pl-9">
                    <span class="flex-shrink-0">
                      <Show
                        when={activeDataset() < 3}
                        fallback={
                          <span class="flex h-10 w-10 items-center justify-center rounded-full bg-fuchsia-600">
                            <FaSolidCheck
                              class="h-6 w-6 fill-current text-white"
                              aria-hidden="true"
                            />
                          </span>
                        }
                      >
                        <span
                          classList={{
                            "flex h-10 w-10 items-center justify-center rounded-full border-2":
                              true,
                            "border-gray-300": activeDataset() == 1,
                            "border-fuchsia-400": activeDataset() == 2,
                          }}
                        >
                          <span
                            classList={{
                              "text-gray-500": activeDataset() == 1,
                              "text-fuchsia-600": activeDataset() == 2,
                            }}
                          >
                            02
                          </span>
                        </span>
                      </Show>
                    </span>
                    <span class="ml-4 mt-0.5 flex min-w-0 flex-col">
                      <span
                        classList={{
                          "text-sm font-medium": true,
                          "text-fuchsia-600": activeDataset() == 2,
                          "text-gray-500": activeDataset() == 1,
                        }}
                      >
                        Add Chunks or Files
                      </span>
                      <span
                        classList={{
                          "text-sm": true,
                          "text-gray-500": activeDataset() < 3,
                        }}
                      >
                        Once you have a Dataset, the next step is including
                        data. You can add Chunks or Upload Files without code
                        using the{" "}
                        <a
                          class="inline underline"
                          href="https://search.trieve.ai"
                        >
                          Search Playground
                        </a>{" "}
                        or programmatically using either the{" "}
                        <a
                          class="inline underline"
                          target="_blank"
                          href="https://docs.trieve.ai/api-reference/chunk/create-or-upsert-chunk-or-chunks"
                        >
                          REST API
                        </a>{" "}
                        ,{" "}
                        <a
                          class="inline underline"
                          target="_blank"
                          href="https://ts-sdk.trieve.ai/functions/Chunk_Methods.createChunk.html"
                        >
                          Typescript SDK
                        </a>
                        ,or{" "}
                        <a
                          class="inline underline"
                          target="_blank"
                          href="https://pypi.org/project/trieve-py-client/"
                        >
                          Python SDK
                        </a>
                        .
                      </span>
                    </span>
                  </span>
                </div>
                {/* <!-- Separator --> */}
                <div
                  class="absolute inset-0 left-0 top-0 hidden w-3 lg:block"
                  aria-hidden="true"
                >
                  <svg
                    class="h-full w-full text-gray-300"
                    viewBox="0 0 12 82"
                    fill="none"
                    preserveAspectRatio="none"
                  >
                    <path
                      d="M0.5 0V31L10.5 41L0.5 51V82"
                      stroke="currentcolor"
                      vector-effect="non-scaling-stroke"
                    />
                  </svg>
                </div>
              </div>
            </li>
            <li class="relative w-full overflow-hidden">
              <div class="w-full overflow-hidden rounded-b-md border border-t-0 border-gray-200 lg:border-0">
                {/* <!-- Upcoming Step --> */}
                <div class="group">
                  <span
                    classList={{
                      "absolute left-0 top-0 h-full w-1 lg:bottom-0 lg:top-auto lg:h-0.5 lg:w-full":
                        true,
                      "bg-fuchsia-600": activeDataset() == 3,
                      "bg-transparent": activeDataset() != 3,
                    }}
                    aria-hidden="true"
                  />
                  <span class="flex items-start px-6 py-5 text-sm lg:pl-9">
                    <span class="flex-shrink-0">
                      <span
                        classList={{
                          "flex h-10 w-10 items-center justify-center rounded-full border-2":
                            true,
                          "border-fuchsia-600": activeDataset() == 3,
                          "border-gray-300": activeDataset() != 3,
                        }}
                      >
                        <span
                          classList={{
                            "text-gray-500": activeDataset() != 3,
                            "text-fuchsia-600": activeDataset() == 3,
                          }}
                        >
                          03
                        </span>
                      </span>
                    </span>
                    <span class="ml-4 mt-0.5 flex min-w-0 flex-col">
                      <span
                        classList={{
                          "text-sm font-medium": true,
                          "text-fuchsia-600": activeDataset() == 3,
                          "text-gray-500": activeDataset() != 3,
                        }}
                      >
                        Evaluate and Tune
                      </span>
                      <span
                        classList={{
                          "text-sm": true,
                          "text-gray-500": activeDataset() < 3,
                        }}
                      >
                        Visit the{" "}
                        <a
                          class="inline underline"
                          href="https://search.trieve.ai"
                        >
                          Search
                        </a>{" "}
                        and{" "}
                        <a
                          class="inline underline"
                          href="https://chat.trieve.ai"
                        >
                          Chat
                        </a>{" "}
                        Playgrounds to evaluate your Dataset's performance with
                        Trieve and make any necessary adjustments before
                        integrating into your application.
                      </span>
                    </span>
                  </span>
                </div>
                {/* <!-- Separator --> */}
                <div
                  class="absolute inset-0 left-0 top-0 hidden w-3 lg:block"
                  aria-hidden="true"
                >
                  <svg
                    class="h-full w-full text-gray-300"
                    viewBox="0 0 12 82"
                    fill="none"
                    preserveAspectRatio="none"
                  >
                    <path
                      d="M0.5 0V31L10.5 41L0.5 51V82"
                      stroke="currentcolor"
                      vector-effect="non-scaling-stroke"
                    />
                  </svg>
                </div>
              </div>
            </li>
          </ol>
        </nav>
      </div>
    </Show>
  );
};
