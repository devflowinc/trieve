import React from "react";
import { Item } from "./item";
import { Chunk, ChunkWithHighlights } from "../utils/types";

export const SearchMode = ({
  results,
  query,
  setQuery,
  setMode,
  onUpOrDownClicked,
  onResultClick,
  showImages,
  placeholder,
  inputRef,
}: {
  results: ChunkWithHighlights[];
  query: string;
  setQuery: (value: string) => void;
  setMode: (value: string) => void;
  onUpOrDownClicked: (index: number, code: string) => void;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  placeholder?: string;
  inputRef: React.RefObject<HTMLInputElement>;
}) => {
  return (
    <>
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
          placeholder={placeholder || "Search for anything"}
        />
        <div className="kbd-wrapper">
          <kbd>ESC</kbd>
        </div>
      </div>
      <ul className="trieve-elements-search">
        {results.length ? (
          <li>
            <button className="item" onClick={() => setMode("chat")}>
              <div className="flex flex-col">
                <h4>Can you tell me about {query}</h4>
                <p className="description">Use AI to answer your question</p>
              </div>
            </button>
          </li>
        ) : null}
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
    </>
  );
};
