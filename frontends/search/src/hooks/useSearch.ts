import { Params, useSearchParams } from "@solidjs/router";
import { createEffect, on } from "solid-js";
import { createStore, unwrap } from "solid-js/store";

export interface SortByField {
  field: string;
}

export interface SortBySearchType {
  rerank_type: string;
  rerank_query: string;
}

export function isSortByField(
  sortBy: SortByField | SortBySearchType,
): sortBy is SortByField {
  return (sortBy as SortByField).field !== undefined;
}

export function isSortBySearchType(
  sortBy: SortByField | SortBySearchType,
): sortBy is SortBySearchType {
  return (sortBy as SortBySearchType).rerank_type !== undefined;
}

export type HighlightStrategy = "v1" | "exactmatch";

export function isHighlightStrategy(
  value: string | undefined,
): value is HighlightStrategy {
  return value === "v1" || value === "exactmatch";
}

export interface SearchOptions {
  version: number;
  query: string;
  searchType: string;
  scoreThreshold: number;
  extendResults: boolean;
  slimChunks: boolean;
  groupUniqueSearch: boolean;
  sort_by: SortByField | SortBySearchType;
  pageSize: number;
  getTotalPages: boolean;
  highlightResults: boolean;
  highlightStrategy: HighlightStrategy;
  highlightThreshold: number;
  highlightDelimiters: string[];
  highlightMaxLength: number;
  highlightMaxNum: number;
  highlightWindow: number;
  group_size: number;
  useQuoteNegatedTerms: boolean;
  removeStopWords: boolean;
}

const initalState: SearchOptions = {
  version: 0, // Variable used to subscribe to entire store.
  query: "",
  searchType: "",
  scoreThreshold: 0.0,
  extendResults: false,
  slimChunks: false,
  groupUniqueSearch: false,
  sort_by: {
    field: "",
  },
  pageSize: 10,
  getTotalPages: false,
  highlightResults: true,
  highlightStrategy: "exactmatch",
  highlightThreshold: 0.8,
  highlightDelimiters: ["?", ".", "!"],
  highlightMaxLength: 8,
  highlightMaxNum: 3,
  highlightWindow: 0,
  group_size: 3,
  useQuoteNegatedTerms: false,
  removeStopWords: false,
};

const fromStateToParams = (state: SearchOptions): Params => {
  return {
    // DO NOT INCLUDE VERSION
    query: state.query,
    searchType: state.searchType,
    scoreThreshold: state.scoreThreshold.toString(),
    extendResults: state.extendResults.toString(),
    slimChunks: state.slimChunks.toString(),
    groupUniqueSearch: state.groupUniqueSearch.toString(),
    sort_by: JSON.stringify(state.sort_by),
    pageSize: state.pageSize.toString(),
    getTotalPages: state.getTotalPages.toString(),
    highlightStrategy: state.highlightStrategy,
    highlightResults: state.highlightResults.toString(),
    highlightThreshold: state.highlightThreshold.toString(),
    highlightDelimiters: state.highlightDelimiters.join(","),
    highlightMaxLength: state.highlightMaxLength.toString(),
    highlightMaxNum: state.highlightMaxNum.toString(),
    highlightWindow: state.highlightWindow.toString(),
    group_size: state.group_size?.toString(),
    useQuoteNegatedTerms: state.useQuoteNegatedTerms.toString(),
    removeStopWords: state.removeStopWords.toString(),
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
    sort_by:
      (JSON.parse(params.sort_by ?? "{}") as SortByField | SortBySearchType) ??
      initalState.sort_by,
    pageSize: parseInt(params.pageSize ?? "10"),
    getTotalPages: (params.getTotalPages ?? "false") === "true",
    highlightResults: (params.highlightResults ?? "true") === "true",
    highlightStrategy: isHighlightStrategy(params.highlightStrategy)
      ? params.highlightStrategy
      : "exactmatch",
    highlightThreshold: parseFloat(params.highlightThreshold ?? "0.8"),
    highlightDelimiters:
      params.highlightDelimiters?.split(",") ?? initalState.highlightDelimiters,
    highlightMaxLength: parseInt(params.highlightMaxLength ?? "8"),
    highlightMaxNum: parseInt(params.highlightMaxNum ?? "3"),
    highlightWindow: parseInt(params.highlightWindow ?? "0"),
    group_size: parseInt(params.group_size ?? "3"),
    useQuoteNegatedTerms: (params.useQuoteNegatedTerms ?? "false") === "true",
    removeStopWords: (params.removeStopWords ?? "false") === "true",
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

  // @ts-expect-error args are passed to setSearch
  const proxiedSet: typeof setSearch = (
    ...args: Parameters<typeof setSearch>
  ) => {
    if (!readFromLocation) return;
    // @ts-expect-error args are passed to setSearch
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
