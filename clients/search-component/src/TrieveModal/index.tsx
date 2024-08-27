import React, { useEffect, useState } from "react";
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
  searchOptions?: Omit<SearchChunksReqPayload, "query">;
  placeholder?: string;
};

export const TrieveModalSearch = ({
  placeholder = "Search...",
  onResultClick,
  showImages,
  trieve,
  theme = "light",
  searchOptions = {
    search_type: "hybrid",
    highlight_options: {
      highlight_delimiters: ["?", ",", ".", "!", "↵"],
      highlight_max_length: 2,
      highlight_max_num: 2,
      highlight_strategy: "exactmatch",
      highlight_window: 100,
    },
  },
}: Props) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<{ chunk: Chunk }[]>([]);

  const search = async () => {
    const results = await trieve.search({
      ...searchOptions,
      query,
    });
    const resultsWithHighlight = results.chunks.map((chunk) => {
      const c = chunk.chunk as unknown as Chunk;
      return {
        ...chunk,
        chunk: {
          ...chunk.chunk,
          highlight: highlightText(query, c.chunk_html),
          highlightTitle: highlightText(
            query,
            c.metadata?.title || c.metadata?.page_title || c.metadata?.name
          ),
          highlightDescription: highlightText(
            query,
            c.metadata?.description || c.metadata?.page_description
          ),
        },
      };
    });
    setResults(resultsWithHighlight as unknown as { chunk: Chunk }[]);
  };
  useEffect(() => {
    if (query) {
      search();
    }
  }, [query]);

  return (
    <Dialog.Root>
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
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
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
        <Dialog.Overlay id="trieve-search-modal-overlay" />
        <Dialog.Content
          id="trieve-search-modal"
          className={theme === "dark" ? "dark" : ""}
        >
          <div className="input-wrapper">
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={placeholder || "sup"}
            />
            <div className="kbd-wrapper">
              <kbd>ESC</kbd>
            </div>
          </div>
          <ul>
            {results.map((result) => (
              <Item
                item={result}
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
