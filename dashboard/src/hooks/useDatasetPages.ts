import { Accessor, createEffect, createMemo, createSignal } from "solid-js";
import { DatasetAndUsage, Organization } from "../types/apiTypes";

const FETCHING_SIZE = 100;
const PAGE_SIZE = 2;

const getDatasets = async ({ orgId }: { orgId: string }) => {
  let page = 0;
  const results: DatasetAndUsage[] = [];
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;
  let canFetchAgain = true;
  while (canFetchAgain) {
    try {
      const params = new URLSearchParams({
        limit: FETCHING_SIZE.toString(),
        offset: (page * FETCHING_SIZE).toString(),
      });
      const response = await fetch(
        `${api_host}/dataset/organization/${orgId}?${params.toString()}`,
        {
          credentials: "include",
          headers: {
            "TR-Organization": orgId,
          },
        },
      );

      if (!response.ok) {
        throw new Error("Failed to fetch datasets");
      }

      const datasets = (await response.json()) as unknown as DatasetAndUsage[];
      if (datasets.length > 0) {
        results.push(...datasets);
        page++;
      } else {
        canFetchAgain = false;
      }
    } catch (error) {
      canFetchAgain = false;
    }
  }
  return results;
};

export const useDatasetPages = (props: {
  org: Accessor<Organization>;
  page: Accessor<number>;
  setPage: (page: number) => void;
}) => {
  const [hasLoaded, setHasLoaded] = createSignal(false);
  const [realDatasets, setRealDatasets] = createSignal<DatasetAndUsage[]>([]);

  createEffect(() => {
    if (!props.org().id) {
      return;
    }
    void getDatasets({ orgId: props.org().id }).then((datasets) => {
      setRealDatasets(datasets);
      setHasLoaded(true);
    });
  });

  const removeDataset = (datasetId: string) => {
    const newDatasets = realDatasets().filter(
      (dataset) => dataset.dataset.id !== datasetId,
    );
    setRealDatasets(newDatasets);
  };

  const currDatasets = createMemo(() => {
    const sliced = realDatasets().slice(
      props.page() * PAGE_SIZE,
      (props.page() + 1) * PAGE_SIZE,
    );
    return sliced;
  });
  const maxPageDiscovered = createMemo(() => {
    return Math.floor(realDatasets().length / PAGE_SIZE);
  });

  return {
    datasets: currDatasets,
    maxPageDiscovered,
    removeDataset,
    hasLoaded,
  };
};
