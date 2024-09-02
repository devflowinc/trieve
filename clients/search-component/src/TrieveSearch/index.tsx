import React, { useState, useEffect, useRef } from "react";
import { useCombobox } from "downshift";
import { Item } from "./Item";
import { Chunk, ChunkWithHighlights, Props } from "../utils/types";
import { throttle } from "lodash-es";
import r2wc from "@r2wc/react-to-web-component";
import { searchWithTrieve } from "../utils/trieve";

export const TrieveSearch = ({
  trieve,
  onResultClick,
  showImages,
  theme = "light",
  placeholder = "Search for anything",
  searchOptions = {
    search_type: "hybrid",
  },
}: Props) => {
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
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
            if (state.selectedItem?.chunk.link) {
              window.open(state.selectedItem?.chunk.link);
            } else {
              onResultClick?.(state.selectedItem?.chunk as Chunk);
            }
            return state;
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
    const results = await searchWithTrieve({
      query: inputValue,
      searchOptions,
      trieve,
    });
    setResults(results);
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

      <ul {...getMenuProps()} className="items-menu">
        {isOpen && results.length ? (
          <>
            <div className="results">
              {results.map((item, index: number) => (
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
          </>
        ) : null}
      </ul>
    </div>
  );
};

export const initTrieveSearch = (props: Props) => {
  const searchWC = r2wc(() => <TrieveSearch {...props} />);

  if (!customElements.get("trieve-search")) {
    customElements.define("trieve-search", searchWC);
  }
};
declare global {
  namespace JSX {
    interface IntrinsicElements {
      "trieve-search": Props;
    }
  }
}
