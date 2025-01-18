import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { Chunk, ChunkWithHighlights, GroupChunk } from "../types";
import {
  ChunkGroup,
  CountChunkQueryResponseBody,
  SearchChunksReqPayload,
  TrieveSDK,
} from "trieve-ts-sdk";
import {
  countChunks,
  countChunksWithPagefind,
  groupSearchWithPagefind,
  groupSearchWithTrieve,
  searchWithPagefind,
  searchWithTrieve,
  getPagefindIndex,
} from "../trieve";

export const ALL_TAG = {
  tag: "all",
  label: "All",
  icon: null,
  iconClassName: "",
};

type simpleSearchReqPayload = Omit<
  SearchChunksReqPayload,
  "query" | "highlight_options"
>;
type customAutoCompleteAddOn = {
  use_autocomplete?: boolean;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type PagefindApi = any;

export type currencyPosition = "before" | "after";
export type ModalTypes = "ecommerce" | "docs" | "pdf";
export type SearchModes = "chat" | "search";
export type searchOptions = simpleSearchReqPayload & customAutoCompleteAddOn;

export interface PagefindOptions {
  usePagefind: boolean;
  cdnBaseUrl?: string;
}

export type ModalProps = {
  datasetId: string;
  apiKey: string;
  baseUrl?: string;
  onResultClick?: (chunk: Chunk) => void;
  theme?: "light" | "dark";
  searchOptions?: searchOptions;
  placeholder?: string;
  chat?: boolean;
  analytics?: boolean;
  ButtonEl?: JSX.ElementType;
  suggestedQueries?: boolean;
  followupQuestions?: boolean;
  numberOfSuggestions?: number;
  defaultSearchQueries?: string[];
  defaultAiQuestions?: string[];
  brandLogoImgSrcUrl?: string;
  brandName?: string;
  problemLink?: string;
  brandColor?: string;
  brandFontFamily?: string;
  openKeyCombination?: { key?: string; label?: string; ctrl?: boolean }[];
  tags?: {
    tag: string;
    label?: string;
    selected?: boolean;
    iconClassName?: string;
    icon?: () => JSX.Element;
  }[];
  defaultSearchMode?: SearchModes;
  usePagefind?: boolean;
  type?: ModalTypes;
  useGroupSearch?: boolean;
  allowSwitchingModes?: boolean;
  defaultCurrency?: string;
  currencyPosition?: currencyPosition;
  responsive?: boolean;
  open?: boolean;
  openLinksInNewTab?: boolean;
  onOpenChange?: (open: boolean) => void;
  debounceMs?: number;
  buttonTriggers?: {
    selector: string;
    mode: SearchModes;
    removeListeners?: boolean;
  }[];
  inline: boolean;
  zIndex?: number;
  showFloatingButton?: boolean;
  floatingButtonPosition?:
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right";
  floatingSearchIconPosition?: "left" | "right";
  showFloatingSearchIcon?: boolean;
  showFloatingInput?: boolean;
  inlineHeader?: string;
  groupTrackingId?: string;
  cleanGroupName?: string;
  cssRelease?: string;
  hideOpenButton?: boolean;
  defaultImageQuestion?: string;
};

const defaultProps = {
  datasetId: "",
  apiKey: "",
  baseUrl: "https://api.trieve.ai",
  defaultSearchMode: "search" as SearchModes,
  placeholder: "Search...",
  theme: "light" as "light" | "dark",
  searchOptions: {
    use_autocomplete: true,
    search_type: "fulltext",
    typo_options: {
      correct_typos: true,
    },
  } as searchOptions,
  analytics: true,
  chat: true,
  suggestedQueries: true,
  followupQuestions: true,
  numberOfSuggestions: 3,
  trieve: (() => {}) as unknown as TrieveSDK,
  openKeyCombination: [{ ctrl: true }, { key: "k", label: "K" }],
  type: "docs" as ModalTypes,
  useGroupSearch: false,
  allowSwitchingModes: true,
  defaultCurrency: "$",
  openLinksInNewTab: false,
  currencyPosition: "before" as currencyPosition,
  responsive: false,
  zIndex: 1000,
  debounceMs: 0,
  showFloatingButton: false,
  floatingButtonPosition: "bottom-right" as
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right",
  floatingSearchIconPosition: "right" as "left" | "right",
  showFloatingSearchIcon: false,
  showFloatingInput: false,
  inline: false,
  inlineHeader: "AI Assistant by Trieve",
  groupTrackingId: undefined,
  cleanGroupName: undefined,
  cssRelease: "stable",
  defaultImageQuestion:
    "This is an image of a product that I want you to show similar recomendations for.",
};

const ModalContext = createContext<{
  props: ModalProps;
  trieveSDK: TrieveSDK;
  query: string;
  imageUrl: string;
  uploadingImage: boolean;
  setQuery: React.Dispatch<React.SetStateAction<string>>;
  setImageUrl: React.Dispatch<React.SetStateAction<string>>;
  setUploadingImage: React.Dispatch<React.SetStateAction<boolean>>;
  results: ChunkWithHighlights[] | GroupChunk[][];
  setResults: React.Dispatch<
    React.SetStateAction<ChunkWithHighlights[] | GroupChunk[][]>
  >;
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
  currentGroup: ChunkGroup | null;
  setCurrentGroup: React.Dispatch<React.SetStateAction<ChunkGroup | null>>;
  tagCounts: CountChunkQueryResponseBody[];
  pagefind?: PagefindApi;
}>({
  props: defaultProps,
  trieveSDK: (() => {}) as unknown as TrieveSDK,
  query: "",
  imageUrl: "",
  uploadingImage: false,
  results: [],
  loadingResults: false,
  open: false,
  inputRef: { current: null },
  modalRef: { current: null },
  mode: "search",
  setMode: () => {},
  setOpen: () => {},
  setQuery: () => {},
  setImageUrl: () => {},
  setUploadingImage: () => {},
  setResults: () => {},
  requestID: "",
  setRequestID: () => {},
  setLoadingResults: () => {},
  setCurrentTag: () => {},
  currentTag: "all",
  currentGroup: null,
  setCurrentGroup: () => {},
  tagCounts: [],
  setContextProps: () => {},
  pagefind: null,
});

const ModalProvider = ({
  children,
  onLoadProps,
}: {
  children: React.ReactNode;
  onLoadProps: ModalProps;
}) => {
  const [props, setProps] = useState<ModalProps>({
    ...defaultProps,
    ...onLoadProps,
  });
  const [query, setQuery] = useState("");
  const [imageUrl, setImageUrl] = useState("");
  const [uploadingImage, setUploadingImage] = useState<boolean>(false);
  const [results, setResults] = useState<
    ChunkWithHighlights[] | GroupChunk[][]
  >([]);
  const [requestID, setRequestID] = useState("");
  const [loadingResults, setLoadingResults] = useState(false);
  const [open, setOpen] = useState(props.open ?? false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState(props.defaultSearchMode || "search");
  const modalRef = useRef<HTMLDivElement>(null);
  const [tagCounts, setTagCounts] = useState<CountChunkQueryResponseBody[]>([]);
  const [currentTag, setCurrentTag] = useState(
    props.tags?.find((t) => t.selected)?.tag || "all"
  );
  const [pagefind, setPagefind] = useState<PagefindApi | null>(null);

  const [currentGroup, setCurrentGroup] = useState<ChunkGroup | null>(null);

  const trieve = new TrieveSDK({
    baseUrl: props.baseUrl,
    apiKey: props.apiKey,
    datasetId: props.datasetId,
    omitCredentials: true,
  });

  const search = async (abortController: AbortController) => {
    if (!query && !imageUrl) {
      setResults([]);
      return;
    }

    try {
      setLoadingResults(true);
      if (props.useGroupSearch && !props.usePagefind) {
        const results = await groupSearchWithTrieve({
          query: query,
          searchOptions: props.searchOptions,
          trieve: trieve,
          abortController,
          ...(currentTag !== "all" && { tag: currentTag }),
          type: props.type,
        });

        const groupMap = new Map<string, GroupChunk[]>();
        results.groups.forEach((group) => {
          const title = group.chunks[0].chunk.metadata?.title;
          if (groupMap.has(title)) {
            groupMap.get(title)?.push(group);
          } else {
            groupMap.set(title, [group]);
          }
        });

        setResults(Array.from(groupMap.values()));
        setRequestID(results.requestID);
      } else if (props.useGroupSearch && props.usePagefind) {
        const results = await groupSearchWithPagefind(
          pagefind,
          query,
          props.datasetId,
          currentTag !== "all" ? currentTag : undefined
        );
        const groupMap = new Map<string, GroupChunk[]>();
        results.groups.forEach((group) => {
          const title = group.chunks[0].chunk.metadata?.title;
          if (groupMap.has(title)) {
            groupMap.get(title)?.push(group);
          } else {
            groupMap.set(title, [group]);
          }
        });
        setResults(Array.from(groupMap.values()));
      } else if (!props.useGroupSearch && props.usePagefind) {
        const results = await searchWithPagefind(
          pagefind,
          query,
          props.datasetId,
          currentTag !== "all" ? currentTag : undefined
        );
        setResults(results);
      } else {
        const results = await searchWithTrieve({
          query_string: query,
          image_url: imageUrl,
          searchOptions: props.searchOptions,
          trieve: trieve,
          abortController,
          ...(currentTag !== "all" && { tag: currentTag }),
          type: props.type,
        });
        setResults(results.chunks);
        setRequestID(results.requestID);
      }
    } catch (e) {
      if ((e as DOMException)?.name != "AbortError") {
        console.error(e);
      }
    } finally {
      setLoadingResults(false);
    }
  };

  const getTagCounts = async (abortController: AbortController) => {
    if (!query) {
      setTagCounts([]);
      return;
    }
    if (props.tags?.length) {
      if (props.usePagefind) {
        const filterCounts = await countChunksWithPagefind(
          pagefind,
          query,
          props.tags
        );
        setTagCounts(filterCounts);
      } else {
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
        } catch (e) {
          if (
            e != "AbortError" &&
            e != "AbortError: signal is aborted without reason"
          ) {
            console.error(e);
          }
        }
      }
    }
  };

  useEffect(() => {
    setProps((p) => ({
      ...p,
      ...onLoadProps,
    }));
  }, [onLoadProps]);

  useEffect(() => {
    if (props.usePagefind) {
      getPagefindIndex(trieve).then((pagefind_base_url) => {
        import(`${pagefind_base_url}/pagefind.js`).then((pagefind) => {
          // @vite-ignore
          setPagefind(pagefind);
          pagefind.filters().then(() => {});
        });
      });
    }
  }, []);

  useEffect(() => {
    props.onOpenChange?.(open);
  }, [open]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (
        open &&
        e.ctrlKey &&
        e.key === "m" &&
        props.allowSwitchingModes !== false
      ) {
        e.preventDefault();
        e.stopPropagation();
        setMode((prevMode) => (prevMode === "chat" ? "search" : "chat"));
      }
    },
    [open, props.allowSwitchingModes]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open, props.allowSwitchingModes]);

  useEffect(() => {
    const abortController = new AbortController();

    const timeout = setTimeout(() => {
      search(abortController);
    }, props.debounceMs);

    return () => {
      clearTimeout(timeout);
      abortController.abort();
    };
  }, [query, imageUrl, currentTag]);

  useEffect(() => {
    const abortController = new AbortController();

    const timeout = setTimeout(() => {
      getTagCounts(abortController);
    }, props.debounceMs);

    return () => {
      clearTimeout(timeout);
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
        imageUrl,
        setImageUrl,
        uploadingImage,
        setUploadingImage,
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
        currentGroup,
        setCurrentGroup,
        tagCounts,
      }}
    >
      {children}
    </ModalContext.Provider>
  );
};

function useModalState() {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useModalState must be used within a ModalProvider");
  }
  return context;
}

export { ModalProvider, useModalState };
