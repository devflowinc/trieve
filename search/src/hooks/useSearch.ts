import { createEffect, on } from "solid-js";
import { createStore, unwrap } from "solid-js/store";

const initalState = {
  version: 0, // Variable used to subscribe to entire store.
  query: "",
  searchType: "",
  scoreThreshold: 0.0,
  extendResults: false,
  slimChunks: false,
  groupUniqueSearch: false,
  recencyBias: 0.0,
  pageSize: 10,
  getTotalPages: true,
  highlightResults: true,
  highlightDelimiters: ["?", ".", "!"],
  highlightMaxLength: 8,
  highlightMaxNum: 3,
  highlightWindow: 0,
};

export const useSearch = () => {
  const [state, setSearch] = createStore(initalState);

  const [debounced, setDebouncedState] = createStore({
    ...initalState,
  });

  createEffect(
    on(
      () => state.version,
      () => {
        const timeout = setTimeout(() => {
          setDebouncedState({ ...unwrap(state) });
        }, 200);
        return () => clearTimeout(timeout);
      },
    ),
  );

  // @ts-expect-error args
  const proxiedSet: typeof setSearch = (
    ...args: Parameters<typeof setSearch>
  ) => {
    setSearch("version", (prev) => prev + 1);
    // @ts-expect-error args
    setSearch(...args);
  };

  return {
    debounced,
    state,
    setSearch: proxiedSet,
  };
};

export type SearchStore = ReturnType<typeof useSearch>;
