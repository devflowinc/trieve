import { Params, useSearchParams } from "@solidjs/router";
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

export type SearchOptions = typeof initalState;

const fromStateToParams = (state: SearchOptions): Params => {
  return {
    // DO NOT INCLUDE VERSION
    query: state.query,
    searchType: state.searchType,
    scoreThreshold: state.scoreThreshold.toString(),
    extendResults: state.extendResults.toString(),
    slimChunks: state.slimChunks.toString(),
    groupUniqueSearch: state.groupUniqueSearch.toString(),
    recencyBias: state.recencyBias.toString(),
    pageSize: state.pageSize.toString(),
    getTotalPages: state.getTotalPages.toString(),
    highlightResults: state.highlightResults.toString(),
    highlightDelimiters: state.highlightDelimiters.join(","),
    highlightMaxLength: state.highlightMaxLength.toString(),
    highlightMaxNum: state.highlightMaxNum.toString(),
    highlightWindow: state.highlightWindow.toString(),
  };
};

const fromParamsToState = (
  params: Partial<Params>,
): Omit<SearchOptions, "version"> => {
  return {
    query: params.query ?? initalState.query,
    searchType: params.searchType ?? initalState.searchType,
    scoreThreshold: parseFloat(params.scoreThreshold ?? "0.0"),
    extendResults: params.extendResults === "true",
    slimChunks: params.slimChunks === "true",
    groupUniqueSearch: params.groupUniqueSearch === "true",
    recencyBias: parseFloat(params.recencyBias ?? "0.0"),
    pageSize: parseInt(params.pageSize ?? "10"),
    getTotalPages: (params.getTotalPages ?? "true") === "true",
    highlightResults: (params.highlightResults ?? "true") === "true",
    highlightDelimiters:
      params.highlightDelimiters?.split(",") ?? initalState.highlightDelimiters,
    highlightMaxLength: parseInt(params.highlightMaxLength ?? "8"),
    highlightMaxNum: parseInt(params.highlightMaxNum ?? "3"),
    highlightWindow: parseInt(params.highlightWindow ?? "0"),
  };
};

let readFromLocation = false;

export const useSearch = () => {
  const [searchParams, setSearchParams] = useSearchParams();

  const [state, setSearch] = createStore({
    ...initalState,
    ...fromParamsToState(searchParams),
  });

  const [debounced, setDebouncedState] = createStore({
    ...initalState,
    ...fromParamsToState(searchParams),
  });

  createEffect(
    on(
      () => state.version,
      () => {
        setSearchParams({
          ...fromStateToParams(unwrap(state)),
        });

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
    if (!readFromLocation) return;
    // @ts-expect-error args
    setSearch(...args);
    setSearch("version", (prev) => prev + 1);
  };

  createEffect(() => {
    const locationState = fromParamsToState(searchParams);
    readFromLocation = true;
    setSearch("searchType", locationState.searchType);
  });

  return {
    debounced,
    state,
    setSearch: proxiedSet,
  };
};

export type SearchStore = ReturnType<typeof useSearch>;
