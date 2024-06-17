import { createStore } from "solid-js/store";

export const useSearch = () => {
  const [state, setSearch] = createStore({
    query: "", // TODO: Debounce
    searchType: "",
    scoreThreshold: 0.0,
    extendResults: false,
    slimChunks: false,
    groupUniqueSearch: false,
    recencyBias: 0.0,
    pageSize: 10,
  });

  return {
    state,
    setSearch,
  };
};

export type SearchStore = ReturnType<typeof useSearch>;
