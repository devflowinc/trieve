import { Accessor, createEffect, createMemo, createSignal } from "solid-js";
import { DatasetAndUsage, Organization } from "../types/apiTypes";
import { createStore } from "solid-js/store";

const PAGE_SIZE = 10;

export const useDatasetPages = (props: {
  org: Accessor<Organization>;
  page: Accessor<number>;
  setPage: (page: number) => void;
}) => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;
  const [hasLoaded, setHasLoaded] = createSignal(false);

  // Prevent rapid clicking while the preloading is happening
  createEffect(() => {
    if (
      pagedDatasets.maxPageDiscovered &&
      props.page() > pagedDatasets.maxPageDiscovered
    ) {
      props.setPage(pagedDatasets.maxPageDiscovered);
    }
  });

  const [pagedDatasets, setPagedDatasets] = createStore({
    datasets: {} as Record<number, DatasetAndUsage[]>,
    maxPageDiscovered: null as number | null,
  });

  createEffect(() => {
    props.org();
    setPagedDatasets("datasets", {});
    setPagedDatasets("maxPageDiscovered", null);
    setHasLoaded(false);
  });

  createEffect(() => {
    if (!props.org().id) {
      return;
    }
    const params = new URLSearchParams({
      limit: PAGE_SIZE.toString(),
      offset: (props.page() * PAGE_SIZE).toString(),
    });
    const page = props.page();
    void fetch(
      `${api_host}/dataset/organization/${props.org().id}?${params.toString()}`,
      {
        credentials: "include",
        headers: {
          "TR-Organization": props.org().id,
        },
      },
    )
      .then((res) => res.json())
      .then((data) => {
        setPagedDatasets("datasets", page, () => {
          return data as DatasetAndUsage[];
        });
        setHasLoaded(true);
      })
      .catch((e) => {
        console.error(e);
        setHasLoaded(true);
      });

    // Prefetch the next page
    const nextParams = new URLSearchParams({
      limit: PAGE_SIZE.toString(),
      offset: ((props.page() + 1) * PAGE_SIZE).toString(),
    });
    void fetch(
      `${api_host}/dataset/organization/${
        props.org().id
      }?${nextParams.toString()}`,
      {
        credentials: "include",
        headers: {
          "TR-Organization": props.org().id,
        },
      },
    )
      .then((res) => res.json())
      .then((data: DatasetAndUsage[]) => {
        if (data.length === 0) {
          setPagedDatasets("maxPageDiscovered", page);
        }
        setPagedDatasets("datasets", page + 1, () => {
          return data;
        });
      });
  });

  const removeDataset = (page: number, datasetId: string) => {
    const maxIndex = Object.keys(pagedDatasets.datasets).length;
    for (let i = page; i < maxIndex; i++) {
      if (i == page) {
        // Remove the dataset and borrow one from the next page
        setPagedDatasets("datasets", i, (datasets) => {
          const removed = datasets.filter((d) => d.dataset.id !== datasetId);
          const toAdd = pagedDatasets.datasets[i + 1]?.at(0);
          if (toAdd) {
            removed.push(toAdd);
          }
          return removed;
        });
      } else {
        // remove the first dataset and borrow from the next one
        setPagedDatasets("datasets", i, (datasets) => {
          if (datasets.length === 0) {
            return [];
          }
          const removed = datasets.slice(1);
          const toAdd = pagedDatasets.datasets[i + 1]?.at(0);
          if (toAdd) {
            removed.push(toAdd);
          }
          return removed;
        });
      }
    }
  };

  const currDatasets = createMemo(() => {
    return pagedDatasets.datasets[props.page()] || [];
  });

  const maxPageDiscovered = createMemo(() => {
    return pagedDatasets.maxPageDiscovered;
  });

  return {
    datasets: currDatasets,
    maxPageDiscovered: maxPageDiscovered,
    removeDataset,
    hasLoaded,
  };
};
