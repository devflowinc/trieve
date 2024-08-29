import React, { act, useEffect, useRef, useState } from "react";
import { SearchChunksReqPayload, TrieveSDK } from "trieve-ts-sdk";
import { Chunk } from "../utils/types";
import * as Dialog from "@radix-ui/react-dialog";
import { highlightText } from "../utils/highlight";
import { Item } from "./item";

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
};

export type ChunkWithHighlights = { chunk: Chunk; highlights: string[] };

export const TrieveModalSearch = ({
  placeholder = "Search...",
  onResultClick,
  showImages,
  trieve,
  theme = "light",
  searchOptions = {
    search_type: "hybrid",
  },
}: Props) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const search = async () => {
    const results = await trieve.search({
      ...searchOptions,
      query,
      highlight_options: {
        highlight_delimiters: ["?", ",", ".", "!", "↵"],
        highlight_max_length: 2,
        highlight_max_num: 1,
        highlight_strategy: "exactmatch",
        highlight_window: 10,
      },
    });
    const resultsWithHighlight = results.chunks.map((chunk) => {
      const c = chunk.chunk as unknown as Chunk;
      return {
        ...chunk,
        chunk: {
          ...chunk.chunk,
          highlight: highlightText(query, c.chunk_html),
        },
      };
    });
    setResults(resultsWithHighlight as unknown as ChunkWithHighlights[]);
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
      index < results.length - 1
        ? document.getElementById(`trieve-search-item-${index + 1}`)?.focus()
        : document.getElementById(`trieve-search-item-0`)?.focus();
    }

    if (code === "ArrowUp") {
      index > 0
        ? document.getElementById(`trieve-search-item-${index - 1}`)?.focus()
        : inputRef.current?.focus();
    }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, []);

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
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
        <Dialog.Overlay id="trieve-search-modal-overlay" />
        <Dialog.Content
          id="trieve-search-modal"
          className={theme === "dark" ? "dark" : ""}
        >
          <div className="input-wrapper">
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
              className="search-icon"
            >
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.3-4.3"></path>
            </svg>
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={placeholder || "sup"}
            />
            <div className="kbd-wrapper">
              <kbd>ESC</kbd>
            </div>
          </div>
          <ul className="trieve-elements-search">
            {results.map((result, index) => (
              <Item
                onUpOrDownClicked={onUpOrDownClicked}
                item={result}
                index={index}
                onResultClick={onResultClick}
                showImages={showImages}
                key={result.chunk.id}
              />
            ))}
          </ul>
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
