import { createQuery } from "@tanstack/solid-query";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { CrawlInterval, CrawlOptions } from "trieve-ts-sdk";
import { createStore } from "solid-js/store";
import { MultiStringInput, Select } from "shared/ui";
import { toTitleCase } from "../../analytics/utils/titleCase";

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
};

export const CrawlingSettings = () => {
  const datasetId = useContext(DatasetContext).datasetId;
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

  return (
    <Show when={crawlSettingsQuery.isSuccess}>
      <RealCrawlingSettings
        mode={crawlSettingsQuery.data ? "edit" : "create"}
        initialCrawlingSettings={crawlSettingsQuery.data || defaultCrawlOptions}
      />
    </Show>
  );
};

interface RealCrawlingSettingsProps {
  initialCrawlingSettings: CrawlOptions;
  mode: "edit" | "create";
}

const Error = (props: { error: string | null | undefined }) => {
  return (
    <Show when={props.error}>
      <div class="text-sm text-red-500">{props.error}</div>
    </Show>
  );
};

type ValidateFn<T extends Record<string, any>> = (value: T) => {
  errors: {
    [key in keyof T]: string | undefined;
  };
  valid: boolean;
};

const RealCrawlingSettings = (props: RealCrawlingSettingsProps) => {
  const [options, setOptions] = createStore(props.initialCrawlingSettings);
  const [errors, setErrors] = createStore<
    ReturnType<ValidateFn<CrawlOptions>>["errors"]
  >({});

  const validate: ValidateFn<CrawlOptions> = (value) => {
    const errors: Record<string, string> = {};
    if (!value.site_url) {
      errors.site_url = "Site URL is required";
    }

    return {
      errors,
      valid: Object.values(errors).filter((v) => !!v).length === 0,
    };
  };

  const submit = () => {
    const validateResult = validate(options);
    if (validateResult.valid) {
      console.log("submit");
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
            onInput={(e) => {
              setOptions("site_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
          <Error error={errors.site_url} />
        </div>
        <div>
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

      <div class="flex items-center gap-2 pt-2">
        <label class="block">Boost Titles</label>
        <input
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
      </div>

      <div class="flex gap-4">
        <div class="pt-2">
          <label class="block" for="">
            Page Limit
          </label>
          <input
            value={options.limit || "0"}
            onInput={(e) => {
              setOptions("limit", parseInt(e.currentTarget.value));
            }}
            class="block rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
        </div>
        <div class="pt-2">
          <label class="block" for="">
            Max Depth
          </label>
          <input
            value={options.max_depth || "0"}
            onInput={(e) => {
              setOptions("max_depth", parseInt(e.currentTarget.value));
            }}
            class="block rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="number"
          />
        </div>
      </div>
      <div class="flex gap-2">
        <div class="pt-4">
          <div>Include Paths</div>
          <MultiStringInput
            addClass="bg-magenta-100/40 px-2 rounded border border-magenta-300/40"
            addLabel="Add Path"
            onChange={(value) => {
              setOptions("include_paths", value);
            }}
            value={options.include_paths || []}
          />
        </div>
        <div class="pt-4">
          <div>Include Tags</div>
          <MultiStringInput
            addClass="bg-magenta-100/40 px-2 rounded border border-magenta-300/40"
            addLabel="Add Tag"
            onChange={(value) => {
              setOptions("include_tags", value);
            }}
            value={options.include_tags || []}
          />
        </div>
      </div>
      <button>Submit</button>
    </form>
  );
};
