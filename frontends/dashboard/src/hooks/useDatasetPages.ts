import { Accessor, createEffect, createMemo, createSignal } from "solid-js";
import createFuzzySearch from "@nozbe/microfuzz";
import { DatasetAndUsage } from "trieve-ts-sdk";

const FETCHING_SIZE = 1000;
const PAGE_SIZE = 20;

const getDatasets = async ({
  orgId,
  onPageFetched,
}: {
  orgId: string;
  onPageFetched?: (page: DatasetAndUsage[]) => void;
}) => {
  let page = 0;
  const results: DatasetAndUsage[] = [];
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;
  let canFetchAgain = true;
  while (canFetchAgain) {
    try {
      const params = new URLSearchParams({
        limit: FETCHING_SIZE.toString(),
        offset: (page * FETCHING_SIZE).toString(),
      });
      const response = await fetch(
        `${apiHost}/dataset/organization/${orgId}?${params.toString()}`,
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
        if (onPageFetched) {
          onPageFetched(datasets);
        }
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
  org: () => { id: string };
  page: Accessor<number>;
  searchQuery: Accessor<string>;
  setPage: (page: number) => void;
}) => {
  const [hasLoaded, setHasLoaded] = createSignal(false);
  const [realDatasets, setRealDatasets] = createSignal<DatasetAndUsage[]>([]);

  createEffect(() => {
    const org_id = props.org().id;

    if (!org_id) {
      return;
    }

    setHasLoaded(false);
    void getDatasets({
      orgId: org_id,
      onPageFetched: (datasets) => {
        setRealDatasets((prevDatasets) => [...prevDatasets, ...datasets]);
        setHasLoaded(true);
      },
    }).then((datasets) => {
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

  const refetchDatasets = async () => {
    const org_id = props.org().id;

    if (!org_id) {
      return;
    }

    await getDatasets({ orgId: org_id }).then((datasets) => {
      setRealDatasets(datasets);
    });
  };

  createEffect(() => {
    void refetchDatasets();
  });

  return {
    datasets: currDatasets,
    maxPageDiscovered,
    maxDatasets,
    removeDataset,
    hasLoaded,
    refetchDatasets,
  };
};
