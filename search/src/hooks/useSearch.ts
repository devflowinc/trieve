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
  getTotalPages: false,
  highlightResults: true,
  highlightDelimiters: ["?", ".", "!"],
  highlightMaxLength: 8,
  highlightMaxNum: 3,
  highlightWindow: 0,
};

export const useSearch = () => {
  const [state, setSearch] = createStore(initalState);

  const [debounced, setDebouncedState] = createStore({
    // Not spreading this results in debouncedState staying
    // perfectly in line with state. Not sure why
    ...initalState,
  });

  createEffect(
    on(
      () => JSON.stringify(state),
      () => {
        console.log("updated");
        const timeout = setTimeout(() => {
          setDebouncedState({ ...unwrap(state) });
        }, 400);
        return () => clearTimeout(timeout);
      },
    ),
  );

  return {
    debounced,
    state,
    setSearch: (...args: Parameters<typeof setSearch>) => {
      setSearch("version", (prev) => prev + 1);
      // @ts-expect-error args
      setSearch(...args);
    },
  };
};

export type SearchStore = ReturnType<typeof useSearch>;
