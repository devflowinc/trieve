import { createQuery } from "@tanstack/solid-query";
import { Show, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { CrawlOptions } from "trieve-ts-sdk";

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
    queryKey: ["crawl-settings", datasetId],
    queryFn: async () => {
      const result = await trieve.fetch(
        "/api/dataset/{dataset_id}/crawl_options",
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
      <div>success</div>
      <RealCrawlingSettings
        initialCrawlingSettings={crawlSettingsQuery.data || defaultCrawlOptions}
      />
    </Show>
  );
};

interface RealCrawlingSettingsProps {
  initialCrawlingSettings: CrawlOptions;
}

const RealCrawlingSettings = (props: RealCrawlingSettingsProps) => {
  return (
    <div>
      <div>Crawling Options</div>
    </div>
  );
};
