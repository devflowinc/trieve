import { createMutation, createQuery } from "@tanstack/solid-query";
import { Show, useContext, createMemo, createEffect } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import {
  CrawlInterval,
  CrawlOpenAPIOptions,
  CrawlOptions,
} from "trieve-ts-sdk";
import { createStore } from "solid-js/store";
import { MultiStringInput, Select } from "shared/ui";
import { toTitleCase } from "../../analytics/utils/titleCase";
import { Spacer } from "../../components/Spacer";
import { UserContext } from "../../contexts/UserContext";
import { createToast } from "../../components/ShowToasts";
import { ValidateFn } from "../../utils/validation";
import { cn } from "shared/utils";

const defaultCrawlOptions: CrawlOptions = {
  boost_titles: false,
  exclude_paths: [],
  exclude_tags: [],
  include_paths: [],
  include_tags: [],
  interval: "daily",
  limit: 1000,
  max_depth: 10,
  site_url: "",
  scrape_options: {},
};

const normalizeOpenAPIOptions = (
  options: CrawlOpenAPIOptions | null | undefined,
) => {
  if (options && options.type == "openapi") {
    if (options.openapi_schema_url === "") {
      return null;
    }
    if (!options.openapi_tag && !options.openapi_schema_url) {
      return null;
    }
    return options;
  } else if (options && options.type == "shopify") {
    return options;
  }
  return null;
};

export const CrawlingSettings = () => {
  const datasetId = useContext(DatasetContext).datasetId;
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const crawlSettingsQuery = createQuery(() => ({
    queryKey: ["crawl-settings", datasetId()],
    queryFn: async () => {
      const result = await trieve.fetch(
        "/api/dataset/crawl_options/{dataset_id}",
        "get",
        {
          datasetId: datasetId(),
        },
      );
      return result.crawl_options ?? null;
    },
  }));

  const updateDatasetMutation = createMutation(() => ({
    mutationKey: ["crawl-settings-update", datasetId()],
    mutationFn: async (options: CrawlOptions) => {
      await trieve.fetch("/api/dataset", "put", {
        data: {
          crawl_options: {
            ...options,
            scrape_options: normalizeOpenAPIOptions(options.scrape_options),
          },
          dataset_id: datasetId(),
        },
        organizationId: userContext.selectedOrg().id,
      });
    },
    onSuccess() {
      createToast({
        title: "Success",
        type: "success",
        message: "Successfully updated crawl options",
      });
    },
    onError() {
      createToast({
        title: "Error",
        type: "error",
        message: "Failed to update crawl options",
      });
    },
  }));

  const onSave = (options: CrawlOptions) => {
    updateDatasetMutation.mutate(options);
  };

  return (
    <Show when={crawlSettingsQuery.isSuccess}>
      <RealCrawlingSettings
        onSave={onSave}
        mode={crawlSettingsQuery.data ? "edit" : "create"}
        initialCrawlingSettings={crawlSettingsQuery.data || defaultCrawlOptions}
      />
    </Show>
  );
};

interface RealCrawlingSettingsProps {
  initialCrawlingSettings: CrawlOptions;
  mode: "edit" | "create";
  onSave: (options: CrawlOptions) => void;
}

const Error = (props: { error: string | null | undefined }) => {
  return (
    <Show when={props.error}>
      <div class="text-sm text-red-500">{props.error}</div>
    </Show>
  );
};

const RealCrawlingSettings = (props: RealCrawlingSettingsProps) => {
  const [options, setOptions] = createStore(props.initialCrawlingSettings);
  const [errors, setErrors] = createStore<
    ReturnType<ValidateFn<CrawlOptions>>["errors"]
  >({});

  const isShopify = createMemo(
    () =>
      options.scrape_options != {} &&
      options.scrape_options?.type === "shopify",
  );
  const isOpenAPI = createMemo(
    () =>
      options.scrape_options?.type != null &&
      options.scrape_options?.type === "openapi",
  );

  createEffect(() => {
    console.log(isOpenAPI());
  });

  const validate: ValidateFn<CrawlOptions> = (value) => {
    const errors: Record<string, string> = {};
    if (!value.site_url) {
      errors.site_url = "Site URL is required";
    }

    if (value.site_url && !value.site_url.startsWith("http")) {
      errors.site_url = "Invalid Site URL - http(s):// required";
    }

    if (value.scrape_options?.type != "shopify") {
      if (!value.limit || value.limit <= 0) {
        errors.limit = "Limit must be greater than 0";
      }
      if (!value.max_depth) {
        errors.max_depth = "Max depth must be greater than 0";
      }
      if (
        value.scrape_options?.type == "openapi" &&
        value.scrape_options?.openapi_tag &&
        !value.scrape_options.openapi_schema_url
      ) {
        errors.scrape_options = "OpenAPI Schema URL is required for tag";
      }
    }

    return {
      errors,
      valid: Object.values(errors).filter((v) => !!v).length === 0,
    };
  };

  const submit = () => {
    const validateResult = validate(options);
    if (validateResult.valid) {
      setErrors({});
      props.onSave(options);
    } else {
      setErrors(validateResult.errors);
    }
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
      class="rounded border border-neutral-300 bg-white p-4 shadow"
    >
      <div class="text-lg">Crawl Options</div>

      <div class="flex w-full items-stretch justify-between gap-4 pt-2">
        <div class="grow">
          <label for="url" class="block">
            Site URL
          </label>
          <input
            name="url"
            value={options.site_url || ""}
            placeholder="URL to crawl..."
            onInput={(e) => {
              setOptions("site_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
          <Error error={errors.site_url} />
        </div>
        <div class="min-w-[200px]">
          <Select
            options={["daily", "weekly", "monthly"] as CrawlInterval[]}
            display={(option) => toTitleCase(option)}
            onSelected={(option) => {
              setOptions("interval", option);
            }}
            class="p-1"
            selected={options.interval || "daily"}
            label="Crawl Interval"
          />
        </div>
      </div>

      <div class="flex items-center gap-2 py-2 pt-4">
        <label class="block">Boost Titles</label>
        <input
          checked={options.boost_titles || false}
          onChange={(e) => {
            setOptions("boost_titles", e.currentTarget.checked);
          }}
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
        <label class="block pl-4">Shopify?</label>
        <input
          onChange={(e) => {
            if (e.currentTarget.checked) {
              setOptions("scrape_options", "type", "shopify");
            } else {
              setOptions("scrape_options", "type", {});
              setOptions("scrape_options", {});
            }
          }}
          checked={isShopify()}
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
        <label class="block pl-4">OpenAPI Spec?</label>
        <input
          onChange={(e) => {
            if (e.currentTarget.checked) {
              setOptions("scrape_options", "type", "openapi");
            } else {
              setOptions("scrape_options", "type", {});
              setOptions("scrape_options", {});
            }
          }}
          checked={isOpenAPI()}
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
      </div>

      <div class={cn("flex gap-4 pt-2", isShopify() && "opacity-40")}>
        <div>
          <label class="block" for="">
            Page Limit
          </label>
          <input
            disabled={isShopify()}
            value={options.limit || "0"}
            onInput={(e) => {
              setOptions("limit", parseInt(e.currentTarget.value));
            }}
            class="block max-w-[100px] rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
          <Error error={errors.limit} />
        </div>
        <div>
          <label class="block" for="">
            Max Depth
          </label>
          <input
            disabled={isShopify()}
            value={options.max_depth || "0"}
            onInput={(e) => {
              setOptions("max_depth", parseInt(e.currentTarget.value));
            }}
            class="block max-w-[100px] rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
          <Error error={errors.max_depth} />
        </div>
        <div class="grow">
          <label class="block" for="">
            OpenAPI Schema URL
          </label>
          <input
            disabled={isShopify() || !isOpenAPI()}
            placeholder="https://example.com/openapi.json"
            value={options.scrape_options?.openapi_schema_url || ""}
            onInput={(e) => {
              if (!options.scrape_options) {
                setOptions("scrape_options", {});
              }
              setOptions(
                "scrape_options",
                "openapi_schema_url",
                e.currentTarget.value,
              );
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
          <Error error={errors.scrape_options} />
        </div>
        <div class="grow">
          <label class="block" for="">
            OpenAPI Tag
          </label>
          <input
            disabled={isShopify() || !isOpenAPI()}
            value={options.scrape_options?.openapi_tag || ""}
            onInput={(e) => {
              if (!options.scrape_options) {
                setOptions("scrape_options", { type: "openapi" });
              }
              setOptions(
                "scrape_options",
                "openapi_tag",
                e.currentTarget.value,
              );
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
      </div>
      <div
        class={cn(
          "grid w-full grid-cols-2 justify-stretch gap-4 pt-4 xl:grid-cols-4",
          isShopify() && "opacity-40",
        )}
      >
        <div class="">
          <div>Include Paths</div>
          <MultiStringInput
            disabled={isShopify()}
            placeholder="/docs/*"
            addClass="bg-magenta-100/40 px-2 rounded text-sm border border-magenta-300/40"
            inputClass="w-full"
            addLabel="Add Path"
            onChange={(value) => {
              setOptions("include_paths", value);
            }}
            value={options.include_paths || []}
          />
          <Error error={errors.include_paths} />
        </div>
        <div class="">
          <div>Exclude Paths</div>
          <MultiStringInput
            disabled={isShopify()}
            placeholder="/admin/*"
            addClass="bg-magenta-100/40 px-2 text-sm rounded border border-magenta-300/40"
            addLabel="Add Path"
            onChange={(value) => {
              setOptions("exclude_paths", value);
            }}
            value={options.exclude_paths || []}
          />
          <Error error={errors.exclude_paths} />
        </div>
        <div class="">
          <div>Include Tags</div>
          <MultiStringInput
            disabled={isShopify()}
            placeholder="h1..."
            addClass="bg-magenta-100/40 text-sm px-2 rounded border border-magenta-300/40"
            addLabel="Add Tag"
            onChange={(value) => {
              setOptions("include_tags", value);
            }}
            value={options.include_tags || []}
          />
          <Error error={errors.include_tags} />
        </div>
        <div class="">
          <div>Exclude Tags</div>
          <MultiStringInput
            disabled={isShopify()}
            placeholder="button..."
            addClass="bg-magenta-100/40 px-2 text-sm rounded border border-magenta-300/40"
            addLabel="Add Tag"
            onChange={(value) => {
              setOptions("exclude_tags", value);
            }}
            value={options.exclude_tags || []}
          />
          <Error error={errors.exclude_tags} />
        </div>
      </div>
      <Spacer h={18} />
      <div class="flex justify-end">
        <button class="rounded border-magenta-200/80 bg-magenta-100 px-3 py-2 font-medium shadow hover:shadow-md">
          Save
        </button>
      </div>
    </form>
  );
};
