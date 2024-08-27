import React, { useState, useEffect, useRef } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import { highlightText } from "../utils/highlight";
import { useCombobox } from "downshift";
import { Item } from "./Item";
import { Chunk } from "../utils/types";
import { ChunkMetadata } from "../../../ts-sdk/dist/types.gen";

type Props = {
  trieve: TrieveSDK;
  searchType: "fulltext" | "semantic" | "hybrid" | "bm25";
  onResultClick: (chunk: Chunk) => void;
  showImages: boolean;
  theme: "light" | "dark";
};

export const TrieveSearch = ({
  trieve,
  searchType,
  onResultClick,
  showImages,
  theme = "light",
}: Props) => {
  const [results, setResults] = useState<{ chunk: Chunk }[]>([]);
  const input = useRef<HTMLDivElement>();
  const { isOpen, getLabelProps, getMenuProps, getInputProps, getItemProps } =
    useCombobox({
      isOpen: true,
      items: results,
      onInputValueChange: ({ inputValue }) => search(inputValue),
    });

  const search = async (inputValue: string) => {
    const results = await trieve.search({
      query: inputValue,
      search_type: searchType || "hybrid",
    });
    const resultsWithHighlight = results.chunks.map((chunk) => ({
      ...chunk,
      chunk: {
        ...chunk.chunk,
        highlight: highlightText(
          inputValue,
          (chunk.chunk as unknown as Chunk).chunk_html
        ),
      },
    }));
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
        Quick search
      </label>
      <div className="input-wrapper" ref={input}>
        <input
          id="search"
          name="search"
          type="text"
          className="search-input"
          placeholder="Search"
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
