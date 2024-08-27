import React, { useState, useEffect, useRef } from "react";
import { TrieveSDK, SearchChunksReqPayload } from "trieve-ts-sdk";
import { highlightText } from "../utils/highlight";
import { useCombobox } from "downshift";
import { Item } from "./Item";
import { Chunk } from "../utils/types";
import { throttle } from "lodash-es";

type Props = {
  trieve: TrieveSDK;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  theme?: "light" | "dark";
  searchOptions?: Omit<SearchChunksReqPayload, "query">;
  placeholder?: string;
};

export const TrieveSearch = ({
  trieve,
  onResultClick,
  showImages,
  theme = "light",
  placeholder = "Search for anything",
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
  const [results, setResults] = useState<{ chunk: Chunk }[]>([]);
  const input = useRef<HTMLDivElement>(null);
  const { isOpen, getLabelProps, getMenuProps, getInputProps, getItemProps } =
    useCombobox({
      items: results,
      onInputValueChange: throttle(
        ({ inputValue }) => search(inputValue),
        1000
      ),
      stateReducer: (state, actionAndChanges) => {
        const { type, changes } = actionAndChanges;
        switch (type) {
          case useCombobox.stateChangeTypes.InputKeyDownEnter: {
            return {
              ...changes,
              inputValue: state.inputValue,
            };
          }
          case useCombobox.stateChangeTypes.InputBlur: {
            return {
              ...changes,
              inputValue: state.inputValue,
            };
          }
          default: {
            return changes;
          }
        }
      },
    });
  const search = async (inputValue: string) => {
    const results = await trieve.search({
      ...searchOptions,
      query: inputValue,
    });
    const resultsWithHighlight = results.chunks.map((chunk) => {
      const c = chunk.chunk as unknown as Chunk;
      return {
        ...chunk,
        chunk: {
          ...chunk.chunk,
          highlight: highlightText(inputValue, c.chunk_html),
          highlightTitle: highlightText(
            inputValue,
            c.metadata?.title || c.metadata?.page_title || c.metadata?.name
          ),
          highlightDescription: highlightText(
            inputValue,
            c.metadata?.description || c.metadata?.page_description
          ),
        },
      };
    });
    setResults(resultsWithHighlight as unknown as { chunk: Chunk }[]);
  };

  const checkForCMDK = (e: KeyboardEvent) => {
    if (e.code === "KeyK" && e.metaKey) {
      input.current?.getElementsByTagName("input")[0].focus();
    }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForCMDK);
    return () => {
      document.removeEventListener("keydown", checkForCMDK);
    };
  });

  return (
    <div
      id="trieve-search-component"
      className={theme === "dark" ? "dark" : ""}
    >
      <label htmlFor="search" className="sr-only" {...getLabelProps()}>
        Search
      </label>
      <div className="input-wrapper" ref={input}>
        <input
          id="search"
          name="search"
          type="text"
          className="search-input"
          placeholder={placeholder}
          {...getInputProps()}
        />
        <div className="kbd-wrapper">
          <kbd>⌘K</kbd>
        </div>
      </div>

      {isOpen && results.length ? (
        <ul {...getMenuProps()} className="items-menu">
          <div className="results">
            {results.map((item: { chunk: Chunk }, index: number) => (
              <Item
                key={`${item.chunk.id}${index}`}
                index={index}
                item={item}
                getItemProps={getItemProps}
                onResultClick={onResultClick}
                showImages={showImages}
              />
            ))}
          </div>
          <li className="trieve-powered">
            <img src="https://cdn.trieve.ai/trieve-logo.png" alt="logo" />
            Powered by Trieve
          </li>
        </ul>
      ) : null}
    </div>
  );
};
