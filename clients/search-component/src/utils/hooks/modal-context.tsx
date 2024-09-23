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
};

const defaultProps = {
  placeholder: "Search...",
  theme: "light" as "light" | "dark",
  searchOptions: {
    search_type: "fulltext",
  } as searchOptions,
  analytics: true,
  chat: true,
  trieve: (() => {}) as unknown as TrieveSDK,
};

const ModalContext = createContext<{
  props: ModalProps;
  query: string;
  setQuery: React.Dispatch<React.SetStateAction<string>>;
  results: ChunkWithHighlights[];
  setResults: React.Dispatch<React.SetStateAction<ChunkWithHighlights[]>>;
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
  props: defaultProps,
  open: false,
  inputRef: { current: null },
  modalRef: { current: null },
  mode: "search",
  setMode: () => {},
  setOpen: () => {},
  setQuery: () => {},
  setResults: () => {},
  setContextProps: () => {},
});

function ModalProvider({ children }: { children: React.ReactNode }) {
  const [props, setProps] = useState<ModalProps>(defaultProps);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState("search");
  const modalRef = useRef<HTMLDivElement>(null);

  const search = async () => {
    const results = await searchWithTrieve({
      query: query,
      searchOptions: props.searchOptions,
      trieve: props.trieve,
    });
    setResults(results);
  };
  useEffect(() => {
    if (query) {
      search();
    }
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
