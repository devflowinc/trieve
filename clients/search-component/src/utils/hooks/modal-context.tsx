import React, {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { Chunk, ChunkWithHighlights } from "../types";
import { AutocompleteReqPayload, TrieveSDK } from "trieve-ts-sdk";
import { searchWithTrieve } from "../trieve";

type searchOptions = Omit<
  Omit<AutocompleteReqPayload, "query">,
  "highlight_options"
>;
export type ModalProps = {
  trieve: TrieveSDK;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  theme?: "light" | "dark";
  searchOptions?: searchOptions;
  placeholder?: string;
  chat?: boolean;
  analytics?: boolean;
  ButtonEl?: JSX.ElementType;
  suggestedQueries?: boolean;
  defaultQueries?: string[];
  openKeyCombination: { key?: string; label?: string; ctrl?: boolean }[];
};

const defaultProps = {
  placeholder: "Search...",
  theme: "light" as "light" | "dark",
  searchOptions: {
    search_type: "fulltext",
  } as searchOptions,
  analytics: true,
  chat: true,
  suggestedQueries: true,
  trieve: (() => {}) as unknown as TrieveSDK,
  defaultQueries: [],
  openKeyCombination: [{ ctrl: true }, { key: "k", label: "K" }],
};

const ModalContext = createContext<{
  props: ModalProps;
  query: string;
  setQuery: React.Dispatch<React.SetStateAction<string>>;
  results: ChunkWithHighlights[];
  setResults: React.Dispatch<React.SetStateAction<ChunkWithHighlights[]>>;
  loadingResults: boolean;
  setLoadingResults: React.Dispatch<React.SetStateAction<boolean>>;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
  inputRef: React.RefObject<HTMLInputElement>;
  mode: string;
  setMode: React.Dispatch<React.SetStateAction<string>>;
  modalRef: React.RefObject<HTMLDivElement>;
  setContextProps: (props: ModalProps) => void;
}>({
  query: "",
  results: [],
  loadingResults: false,
  props: defaultProps,
  open: false,
  inputRef: { current: null },
  modalRef: { current: null },
  mode: "search",
  setMode: () => {},
  setOpen: () => {},
  setQuery: () => {},
  setResults: () => {},
  setLoadingResults: () => {},
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
  const [loadingResults, setLoadingResults] = useState(false);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState("search");
  const modalRef = useRef<HTMLDivElement>(null);

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

    setLoadingResults(true);

    try {
      const results = await searchWithTrieve({
        query: query,
        searchOptions: props.searchOptions,
        trieve: props.trieve,
        abortController,
      });
      setResults(results);
    } catch (e) {
      console.error(e);
    }

    setLoadingResults(false);
  };

  useEffect(() => {
    const abortController = new AbortController();

    search(abortController);

    return () => {
      abortController.abort();
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
        query,
        setQuery,
        open,
        setOpen,
        inputRef,
        results,
        setResults,
        loadingResults,
        setLoadingResults,
        mode,
        setMode,
        modalRef,
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
