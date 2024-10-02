import { createQuery } from "@tanstack/solid-query";
import { useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";

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
    <div>
      <div>Crawling settings</div>
    </div>
  );
};
