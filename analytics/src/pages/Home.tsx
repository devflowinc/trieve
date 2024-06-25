import { useContext } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { DatasetContext } from "../layouts/TopBarLayout";
import { apiHost } from "../utils/apiHost";
import { DatasetUsageCount } from "shared/types";

export const Home = () => {
  const dataset = useContext(DatasetContext);
  const datasetUsageQuery = createQuery(() => ({
    queryKey: ["dataset-usage", dataset()?.dataset.id],
    queryFn: async () => {
      const response = await fetch(
        `${apiHost}/dataset/usage/${dataset().dataset.id}`,
        {
          method: "GET",
          headers: {
            "TR-Dataset": dataset().dataset.id,
            "Content-Type": "application/json",
          },
          credentials: "include",
        },
      );
      if (!response.ok) {
        throw new Error("Failed to fetch dataset usage count");
      }
      return (await response.json()) as DatasetUsageCount;
    },
  }));

  return (
    <div>
      <div>Home Page</div>
      <div>Dataset Chunks: {datasetUsageQuery.data?.chunk_count}</div>
    </div>
  );
};
