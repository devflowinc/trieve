import React, { useEffect, useRef, useState } from "react";
import { SearchChunksReqPayload, TrieveSDK } from "trieve-ts-sdk";
import { Chunk, ChunkWithHighlights } from "../utils/types";
import * as Dialog from "@radix-ui/react-dialog";
import r2wc from "@r2wc/react-to-web-component";
import { searchWithTrieve } from "../utils/trieve";
import { SearchMode } from "./SearchMode";
import { ChatMode } from "./ChatMode";

type Props = {
  trieve: TrieveSDK;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  theme?: "light" | "dark";
  searchOptions?: Omit<
    Omit<SearchChunksReqPayload, "query">,
    "highlight_options"
  >;
  placeholder?: string;
  chat?: boolean;
};

export const TrieveModalSearch = ({
  placeholder = "Search...",
  onResultClick,
  showImages,
  trieve,
  theme = "light",
  searchOptions = {
    search_type: "hybrid",
  },
  chat = true,
}: Props) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState("search");
  const modalRef = useRef<HTMLDivElement>(null);

  const search = async () => {
    const results = await searchWithTrieve({
      query,
      searchOptions,
      trieve,
    });
    setResults(results);
  };
  useEffect(() => {
    if (query) {
      search();
    }
  }, [query]);

  const checkForInteractions = (e: KeyboardEvent) => {
    if (e.code === "KeyK" && e.metaKey && !open) setOpen(true);
    if (e.code === "ArrowDown" && inputRef.current === document.activeElement) {
      document.getElementById(`trieve-search-item-0`)?.focus();
    }
  };

  const onUpOrDownClicked = (index: number, code: string) => {
    if (code === "ArrowDown") {
      if (index < results.length - 1) {
        document.getElementById(`trieve-search-item-${index + 1}`)?.focus();
      } else {
        document.getElementById(`trieve-search-item-0`)?.focus();
      }
    }

    if (code === "ArrowUp") {
      if (index > 0) {
        document.getElementById(`trieve-search-item-${index - 1}`)?.focus();
      } else {
        inputRef.current?.focus();
      }
    }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, []);

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(value) => {
        setOpen(value);
        setMode("search");
      }}
    >
      <Dialog.Trigger asChild>
        <button
          id="open-trieve-modal"
          type="button"
          className={theme === "dark" ? "dark" : ""}
        >
          <div>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.3-4.3"></path>
            </svg>
            <div>{placeholder}</div>
          </div>
          <span className="open">⌘K</span>
        </button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.DialogTitle className="sr-only">Search</Dialog.DialogTitle>
        <Dialog.DialogDescription className="sr-only">
          Search or ask an AI
        </Dialog.DialogDescription>
        <Dialog.Overlay id="trieve-search-modal-overlay" />
        <Dialog.Content
          id="trieve-search-modal"
          ref={modalRef}
          className={theme === "dark" ? "dark" : ""}
          style={{ overflow: "auto" }}
        >
          {mode === "search" ? (
            <SearchMode
              results={results}
              setQuery={setQuery}
              query={query}
              setMode={setMode}
              onUpOrDownClicked={onUpOrDownClicked}
              showImages={showImages}
              onResultClick={onResultClick}
              placeholder={placeholder}
              inputRef={inputRef}
              chat={chat}
            />
          ) : (
            <ChatMode
              onNewMessage={() =>
                modalRef.current?.scroll({ top: 0, behavior: "smooth" })
              }
              query={query}
              setMode={setMode}
              trieve={trieve}
            />
          )}
          <a
            className="trieve-powered"
            href="https://trieve.ai"
            target="_blank"
          >
            <img src="https://cdn.trieve.ai/trieve-logo.png" alt="logo" />
            Powered by Trieve
          </a>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export const initTrieveModalSearch = (props: Props) => {
  const ModalSearchWC = r2wc(() => <TrieveModalSearch {...props} />);

  if (!customElements.get("trieve-modal-search")) {
    customElements.define("trieve-modal-search", ModalSearchWC);
  }
};
