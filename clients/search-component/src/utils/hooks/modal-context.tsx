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
  ChunkFilter,
  ChunkGroup,
  SearchChunksReqPayload,
  TrieveSDK,
} from "trieve-ts-sdk";
import {
  groupSearchWithPagefind,
  groupSearchWithTrieve,
  searchWithPagefind,
  searchWithTrieve,
  getPagefindIndex,
} from "../trieve";
import { InferenceFilterFormStep } from "../../TrieveModal/FilterSidebarComponents";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";

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

export interface TagProp {
  tag: string;
  label?: string;
  selected?: boolean;
  iconClassName?: string;
  icon?: () => JSX.Element;
  description?: string;
  child?: FilterSidebarSection;
}

export interface FilterSidebarSection {
  key: string;
  title: string;
  selectionType: "single" | "multiple";
  filterType: "match_any" | "match_all";
  options: TagProp[];
}

export interface FilterSidebarProps {
  sections: FilterSidebarSection[];
}

export interface InferenceFiltersFormProps {
  steps: InferenceFilterFormStep[];
}

export interface SearchPageProps {
  filterSidebarProps?: FilterSidebarProps;
  inferenceFiltersFormProps?: InferenceFiltersFormProps;
  display?: boolean;
}

export type ModalProps = {
  datasetId: string;
  apiKey: string;
  partnerSettings?: {
    partnerCompanyName?: string;
    partnerCompanyUrl?: string;
    partnerCompanyFaviconUrl?: string;
  };
  baseUrl?: string;
  onResultClick?: (chunk: Chunk) => void;
  theme?: "light" | "dark";
  searchOptions?: searchOptions;
  chatFilters?: ChunkFilter;
  placeholder?: string;
  chatPlaceholder?: string;
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
  tags?: TagProp[];
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
  inline?: boolean;
  inlineCarousel?: boolean;
  zIndex?: number;
  showFloatingButton?: boolean;
  floatingButtonPosition?:
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right";
  floatingButtonVersion?: "brand-logo" | "brand-color";
  floatingSearchIconPosition?: "left" | "right";
  showFloatingSearchIcon?: boolean;
  disableFloatingSearchIconClick?: boolean;
  showFloatingInput?: boolean;
  inlineHeader?: string;
  groupTrackingId?: string;
  cleanGroupName?: string;
  cssRelease?: string;
  hideOpenButton?: boolean;
  defaultImageQuestion?: string;
  onAddToCart?: (chunk: Chunk) => Promise<void> | void;
  getCartQuantity?: (trackingId: string) => Promise<number> | number;
  showResultHighlights?: boolean;
  initialAiMessage?: string;
  ignoreEventListeners?: boolean;
  hideOverlay?: boolean;
  hidePrice?: boolean;
  hideChunkHtml?: boolean;
  componentName?: string;
  displayModal?: boolean;
  searchPageProps?: SearchPageProps;
  recommendOptions?: {
    queriesToTriggerRecommendations: string[];
    productId: string;
    filter?: ChunkFilter;
  };
  usePortal?: boolean;
  previewTopicId?: string;
};

const defaultProps = {
  datasetId: "",
  apiKey: "",
  baseUrl: "https://api.trieve.ai",
  defaultSearchMode: "search" as SearchModes,
  placeholder: "Search...",
  chatPlaceholder: "Ask Anything...",
  theme: "light" as "light" | "dark",
  searchOptions: {
    use_autocomplete: true,
    search_type: "fulltext",
    typo_options: {
      correct_typos: true,
    },
  } as searchOptions,
  chatFilters: undefined,
  analytics: true,
  chat: true,
  suggestedQueries: true,
  followupQuestions: true,
  numberOfSuggestions: 3,
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
  floatingButtonVersion: "brand-logo" as "brand-logo" | "brand-color",
  floatingSearchIconPosition: "right" as "left" | "right",
  showFloatingSearchIcon: false,
  disableFloatingSearchIconClick: false,
  showFloatingInput: false,
  inline: false,
  inlineCarousel: false,
  inlineHeader: "AI Assistant by Trieve",
  groupTrackingId: undefined,
  cleanGroupName: undefined,
  cssRelease: "stable",
  defaultImageQuestion:
    "This is an image of a product that I want you to show similar recomendations for.",
  onAddToCart: undefined,
  showResultHighlights: true,
  initialAiMessage: undefined,
  ignoreEventListeners: false,
  hideOverlay: false,
  hidePrice: false,
  hideChunkHtml: false,
  componentName: "trieve-modal-search",
  displayModal: true,
  searchPageProps: {
    filterSidebarProps: {
      sections: [],
    } as FilterSidebarProps,
  } as SearchPageProps,
  usePortal: true,
  previewTopicId: undefined,
} satisfies ModalProps;

const ModalContext = createContext<{
  props: ModalProps;
  trieveSDK: TrieveSDK;
  query: string;
  imageUrl: string;
  audioBase64: string | undefined;
  uploadingImage: boolean;
  fingerprint: string;
  setFingerprint: React.Dispatch<React.SetStateAction<string>>;
  setQuery: React.Dispatch<React.SetStateAction<string>>;
  setImageUrl: React.Dispatch<React.SetStateAction<string>>;
  setAudioBase64: React.Dispatch<React.SetStateAction<string | undefined>>;
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
  selectedTags: TagProp[];
  setSelectedTags: React.Dispatch<React.SetStateAction<TagProp[] | undefined>>;
  currentGroup: ChunkGroup | null;
  setCurrentGroup: React.Dispatch<React.SetStateAction<ChunkGroup | null>>;
  pagefind?: PagefindApi;
  isRecording: boolean;
  setIsRecording: React.Dispatch<React.SetStateAction<boolean>>;
  // sidebar filter specific state
  selectedSidebarFilters: Record<string, string[]>; // hashmap where key is the section key and value are the selected labels
  setSelectedSidebarFilters: React.Dispatch<
    React.SetStateAction<Record<string, string[]>>
  >;
  minHeight: number;
  resetHeight: () => void;
  addHeight: (height: number) => void;
}>({
  props: defaultProps,
  trieveSDK: (() => {}) as unknown as TrieveSDK,
  query: "",
  imageUrl: "",
  audioBase64: "",
  uploadingImage: false,
  results: [],
  loadingResults: false,
  open: false,
  inputRef: { current: null },
  modalRef: { current: null },
  mode: "search",
  fingerprint: "",
  setFingerprint: () => {},
  setMode: () => {},
  setOpen: () => {},
  setQuery: () => {},
  setImageUrl: () => {},
  setAudioBase64: () => {},
  setUploadingImage: () => {},
  setResults: () => {},
  requestID: "",
  setRequestID: () => {},
  setLoadingResults: () => {},
  selectedTags: [],
  setSelectedTags: () => {},
  currentGroup: null,
  setCurrentGroup: () => {},
  setContextProps: () => {},
  pagefind: null,
  isRecording: false,
  setIsRecording: () => {},
  // sidebar filter specific state
  selectedSidebarFilters: {},
  setSelectedSidebarFilters: () => {},
  minHeight: 0,
  resetHeight: () => {},
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  addHeight: (height: number) => {},
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
  const [fingerprint, setFingerprint] = useState("");
  const [imageUrl, setImageUrl] = useState("");
  const [audioBase64, setAudioBase64] = useState<string | undefined>(undefined);
  const [isRecording, setIsRecording] = useState(false);
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
  const [selectedTags, setSelectedTags] = useState(
    props.tags?.filter((t) => t.selected),
  );
  const [pagefind, setPagefind] = useState<PagefindApi | null>(null);
  const [currentGroup, setCurrentGroup] = useState<ChunkGroup | null>(null);
  const [selectedSidebarFilters, setSelectedSidebarFilters] = useState<
    Record<string, string[]>
  >({});
  const [minHeight, setMinHeight] = useState(0);
  const [chatHeight, setChatHeight] = useState(0);
  const [enabled, setEnabled] = useState(true);

  const trieve = new TrieveSDK({
    baseUrl: props.baseUrl,
    apiKey: props.apiKey,
    datasetId: props.datasetId,
    omitCredentials: true,
  });

  const search = async (abortController: AbortController) => {
    if (!query && !imageUrl && !audioBase64) {
      setResults([]);
      return;
    }

    try {
      setLoadingResults(true);
      if (props.useGroupSearch && !props.usePagefind) {
        const results = await groupSearchWithTrieve({
          props,
          query_string: query,
          image_url: imageUrl,
          audioBase64: audioBase64,
          searchOptions: props.searchOptions,
          trieve: trieve,
          abortController,
          ...(selectedTags?.map((t) => t.tag) ?? []),
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

        if (results.transcribedQuery && audioBase64) {
          setQuery(results.transcribedQuery);
          setAudioBase64(undefined);
        }
        setResults(Array.from(groupMap.values()));
        setRequestID(results.requestID);
      } else if (props.useGroupSearch && props.usePagefind) {
        const results = await groupSearchWithPagefind(
          pagefind,
          query,
          props.datasetId,
          selectedTags?.map((t) => t.tag),
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
          selectedTags?.map((t) => t.tag),
        );
        setResults(results);
      } else {
        const results = await searchWithTrieve({
          props,
          query_string: query,
          image_url: imageUrl,
          audioBase64: audioBase64,
          searchOptions: props.searchOptions,
          trieve: trieve,
          abortController,
          tags: selectedTags?.map((t) => t.tag),
          type: props.type,
        });
        if (results.transcribedQuery && audioBase64) {
          setQuery(results.transcribedQuery);
          setAudioBase64(undefined);
        }
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
    getFingerprint().then((fingerprint) => {
      setFingerprint(fingerprint);
    });
  }, []);

  useEffect(() => {
    props.onOpenChange?.(open);
  }, [open]);

  useEffect(() => {
    const abortController = new AbortController();

    if (open && props.analytics && props.previewTopicId === undefined) {
      try {
        getFingerprint().then((fingerprint) => {
          trieve.sendAnalyticsEvent(
            {
              event_name: `component_open`,
              event_type: "click",
              clicked_items: null,
              user_id: fingerprint,
              location: window.location.href,
              metadata: {
                component_props: props,
              },
            },
            abortController.signal,
          );
        });
      } catch (e) {
        console.log("error on click event", e);
      }
    }

    return () => {
      abortController.abort("AbortError on component_open");
    };
  }, [open, props.analytics, props]);

  useEffect(() => {
    const abortController = new AbortController();

    if (!open && props.analytics && props.previewTopicId === undefined) {
      try {
        getFingerprint().then((fingerprint) => {
          trieve.sendAnalyticsEvent(
            {
              event_name: `component_close`,
              event_type: "click",
              clicked_items: null,
              user_id: fingerprint,
              location: window.location.href,
              metadata: {
                component_props: props,
              },
            },
            abortController.signal,
          );
        });
      } catch (e) {
        console.log("error on click event", e);
      }
    }

    return () => {
      abortController.abort("AbortError on component_close");
    };
  }, [open, props.analytics, props]);

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
    [open, props.allowSwitchingModes],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open, props.allowSwitchingModes]);

  useEffect(() => {
    if (mode != "search") return;

    const abortController = new AbortController();

    const timeout = setTimeout(() => {
      search(abortController);
    }, props.debounceMs);

    return () => {
      clearTimeout(timeout);
      abortController.abort();
    };
  }, [query, imageUrl, audioBase64, selectedTags, mode]);

  useEffect(() => {
    if (!modalRef || !modalRef.current) {
      return;
    }
    const ref = modalRef.current;
    const observer = new ResizeObserver((entries) => {
      setChatHeight(entries[0].contentRect.height);
    });

    observer.observe(ref);
    return () => {
      observer.disconnect();
    };
  }, [modalRef]);

  useEffect(() => {
    if (chatHeight > minHeight && enabled) {
      setMinHeight(chatHeight);
    }
  }, [chatHeight, minHeight, enabled]);

  const resetHeight = useCallback(() => {
    setMinHeight(0);
    setEnabled(false);
    setTimeout(() => {
      setEnabled(true);
    }, 200);
  }, []);

  const addHeight = useCallback((height: number) => {
    setMinHeight((prev) => prev + height);
  }, []);

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
        audioBase64,
        setAudioBase64,
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
        selectedTags: selectedTags ?? [],
        setSelectedTags,
        currentGroup,
        setCurrentGroup,
        isRecording,
        setIsRecording,
        selectedSidebarFilters,
        setSelectedSidebarFilters,
        fingerprint,
        setFingerprint,
        minHeight,
        resetHeight,
        addHeight,
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
