import { createStore } from "solid-js/store";

export const useSearch = () => {
  const [state, setSearch] = createStore({
    query: "", // TODO: Debounce
  });

  return {
    state,
    setSearch,
  };
};

export type SearchStore = ReturnType<typeof useSearch>;
