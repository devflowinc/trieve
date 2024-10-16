import React, {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { Chunk, ChunkWithHighlights } from "../types";
import {
  AutocompleteReqPayload,
  CountChunkQueryResponseBody,
  TrieveSDK,
} from "trieve-ts-sdk";
import { countChunks, searchWithTrieve } from "../trieve";

export const ALL_TAG = { tag: "all", label: "All", icon: null };

type currencyPosition = "before" | "after";
type ModalTypes = "ecommerce" | "docs";
type SearchModes = "chat" | "search";
type searchOptions = Omit<
  Omit<AutocompleteReqPayload, "query">,
  "highlight_options"
>;
export type ModalProps = {
  datasetId: string;
  apiKey: string;
  onResultClick?: (chunk: Chunk) => void;
  theme?: "light" | "dark";
  searchOptions?: searchOptions;
  placeholder?: string;
  chat?: boolean;
  analytics?: boolean;
  ButtonEl?: JSX.ElementType;
  suggestedQueries?: boolean;
  defaultSearchQueries?: string[];
  defaultAiQuestions?: string[];
  brandLogoImgSrcUrl?: string;
  brandName?: string;
  brandColor?: string;
  openKeyCombination?: { key?: string; label?: string; ctrl?: boolean }[];
  tags?: {
    tag: string;
    label?: string;
    selected?: boolean;
    icon?: () => JSX.Element;
  }[];
  defaultSearchMode?: SearchModes;
  type?: ModalTypes;
  allowSwitchingModes?: boolean;
  defaultCurrency?: string;
  currencyPosition?: currencyPosition;
};

const defaultProps = {
  datasetId: "",
  apiKey: "",
  defaultSearchMode: "search" as SearchModes,
  placeholder: "Search...",
  theme: "light" as "light" | "dark",
  searchOptions: {
    search_type: "fulltext",
  } as searchOptions,
  analytics: true,
  chat: true,
  suggestedQueries: true,
  trieve: (() => {}) as unknown as TrieveSDK,
  openKeyCombination: [{ ctrl: true }, { key: "k", label: "K" }],
  type: "docs" as ModalTypes,
  allowSwitchingModes: true,
  currencyPosition: "after" as currencyPosition,
};

const ModalContext = createContext<{
  props: ModalProps;
  trieveSDK: TrieveSDK;
  query: string;
  setQuery: React.Dispatch<React.SetStateAction<string>>;
  results: ChunkWithHighlights[];
  setResults: React.Dispatch<React.SetStateAction<ChunkWithHighlights[]>>;
  requestID: string;
  setRequestID: React.Dispatch<React.SetStateAction<string>>;
  loadingResults: boolean;
  setLoadingResults: React.Dispatch<React.SetStateAction<boolean>>;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
  inputRef: React.RefObject<HTMLInputElement>;
  mode: string;
  setMode: React.Dispatch<React.SetStateAction<SearchModes>>;
  modalRef: React.RefObject<HTMLDivElement>;
  setContextProps: (props: ModalProps) => void;
  currentTag: string;
  setCurrentTag: React.Dispatch<React.SetStateAction<string>>;
  tagCounts: CountChunkQueryResponseBody[];
}>({
  props: defaultProps,
  trieveSDK: (() => {}) as unknown as TrieveSDK,
  query: "",
  results: [],
  loadingResults: false,
  open: false,
  inputRef: { current: null },
  modalRef: { current: null },
  mode: "search",
  setMode: () => {},
  setOpen: () => {},
  setQuery: () => {},
  setResults: () => {},
  requestID: "",
  setRequestID: () => {},
  setLoadingResults: () => {},
  setCurrentTag: () => {},
  currentTag: "all",
  tagCounts: [],
  setContextProps: () => {},
});

function ModalProvider({
  children,
  onLoadProps,
}: {
  children: React.ReactNode;
  onLoadProps: ModalProps;
}) {
  const [props, setProps] = useState<ModalProps>({
    ...defaultProps,
    ...onLoadProps,
  });
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
  const [requestID, setRequestID] = useState("");
  const [loadingResults, setLoadingResults] = useState(false);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState(props.defaultSearchMode || "search");
  const modalRef = useRef<HTMLDivElement>(null);
  const [tagCounts, setTagCounts] = useState<CountChunkQueryResponseBody[]>([]);
  const [currentTag, setCurrentTag] = useState(
    props.tags?.find((t) => t.selected)?.tag || "all"
  );

  const trieve = new TrieveSDK({
    apiKey: props.apiKey,
    datasetId: props.datasetId,
  });

  useEffect(() => {
    setProps((p) => ({
      ...p,
      ...onLoadProps,
    }));
  }, [onLoadProps]);

  const search = async (abortController: AbortController) => {
    if (!query) {
      setResults([]);
      return;
    }

    try {
      setLoadingResults(true);
      const results = await searchWithTrieve({
        query: query,
        searchOptions: props.searchOptions,
        trieve: trieve,
        abortController,
        ...(currentTag !== "all" && { tag: currentTag }),
      });
      setResults(results.chunks);
      setRequestID(results.requestID);
    } catch (e) {
      if (e != 'AbortError' && e != "AbortError: signal is aborted without reason") {
        console.error(e);
      }
    } finally {
      setLoadingResults(false);
    }
  };

  useEffect(() => {
    const abortController = new AbortController();
    search(abortController);

    return () => {
      abortController.abort();
    };
  }, [query, currentTag]);

  const getTagCounts = async (abortController: AbortController) => {
    if (!query) {
      setTagCounts([]);
      return;
    }
    if (props.tags?.length) {
      try {
        const numberOfRecords = await Promise.all(
          [ALL_TAG, ...props.tags].map((tag) =>
            countChunks({
              query: query,
              trieve: trieve,
              abortController,
              ...(tag.tag !== "all" && { tag: tag.tag }),
            })
          )
        );
        setTagCounts(numberOfRecords);
      } catch(e) {
        if (e != 'AbortError' && e != "AbortError: signal is aborted without reason") {
          console.log(e);
          console.log(typeof e);
          console.error(e);
        }
      }
    }
  };

  useEffect(() => {
    const abortController = new AbortController();
    getTagCounts(abortController);

    return () => {
      abortController.abort("AbortError");
    };
  }, [query]);

  return (
    <ModalContext.Provider
      value={{
        setContextProps: (props) =>
          setProps((p) => ({
            ...p,
            ...props,
          })),
        props,
        trieveSDK: trieve,
        query,
        setQuery,
        open,
        setOpen,
        inputRef,
        results,
        setResults,
        requestID,
        setRequestID,
        loadingResults,
        setLoadingResults,
        mode,
        setMode,
        modalRef,
        currentTag,
        setCurrentTag,
        tagCounts,
      }}
    >
      {children}
    </ModalContext.Provider>
  );
}

function useModalState() {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useModalState must be used within a ModalProvider");
  }
  return context;
}

export { ModalProvider, useModalState };
