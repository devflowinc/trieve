import { createQuery } from "@tanstack/solid-query";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { CrawlInterval, CrawlOptions } from "trieve-ts-sdk";
import { createStore } from "solid-js/store";
import { Select } from "shared/ui";
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

const RealCrawlingSettings = (props: RealCrawlingSettingsProps) => {
  const [options, setOptions] = createStore(props.initialCrawlingSettings);

  return (
    <div class="rounded border border-neutral-300 bg-white p-4 shadow">
      <div class="text-lg">Crawl Options</div>

      <div class="flex w-full items-stretch justify-between gap-4">
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
            class="block w-full rounded-md border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
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

      <div>
        <label for="">Max Pages</label>
        <input type="number"></input>
      </div>
    </div>
  );
};
