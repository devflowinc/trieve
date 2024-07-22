import { Accessor, createEffect, createMemo, createSignal } from "solid-js";
import { DatasetAndUsage, Organization } from "shared/types";
import createFuzzySearch from "@nozbe/microfuzz";

const FETCHING_SIZE = 1000;
const PAGE_SIZE = 20;

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
    } catch (_error) {
      canFetchAgain = false;
    }
  }

  return results;
};

export const useDatasetPages = (props: {
  org: Accessor<Organization | undefined>;
  page: Accessor<number>;
  searchQuery: Accessor<string>;
  setPage: (page: number) => void;
}) => {
  const [hasLoaded, setHasLoaded] = createSignal(false);
  const [realDatasets, setRealDatasets] = createSignal<DatasetAndUsage[]>([]);

  createEffect(() => {
    const org_id = props.org()?.id;

    if (!org_id) {
      return;
    }

    setHasLoaded(false);
    void getDatasets({ orgId: org_id }).then((datasets) => {
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
    let startingSet = realDatasets();

    if (props.searchQuery() && props.searchQuery() !== "") {
      const fuzzy = createFuzzySearch(startingSet, {
        getText(datasetAndUsage: DatasetAndUsage) {
          return [datasetAndUsage.dataset.name, datasetAndUsage.dataset.id];
        },
      });
      startingSet = fuzzy(props.searchQuery()).map((result) => result.item);
    }

    const sliced = startingSet.slice(
      props.page() * PAGE_SIZE,
      (props.page() + 1) * PAGE_SIZE,
    );
    return sliced;
  });

  const maxPageDiscovered = createMemo(() => {
    return Math.floor(realDatasets().length / PAGE_SIZE);
  });

  const maxDatasets = createMemo(() => {
    return realDatasets().length;
  });

  return {
    datasets: currDatasets,
    maxPageDiscovered,
    maxDatasets,
    removeDataset,
    hasLoaded,
  };
};
