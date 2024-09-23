import React from "react";
import { Item } from "./item";
import { AIIcon, ArrowIcon, LoadingIcon } from "./icons";
import { useSuggestedQueries } from "../utils/hooks/useSuggestedQueries";
import { useModalState } from "../utils/hooks/modal-context";

export const SearchMode = () => {
  const { props, results, query, setQuery, inputRef, setMode } =
    useModalState();
  const { suggestedQueries, isFetchingSuggestedQueries } =
    useSuggestedQueries();

  return (
    <>
      <div className="input-wrapper">
        <div className="input-flex">
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
            placeholder={props.placeholder || "Search for anything"}
          />

          <div className="kbd-wrapper">
            <kbd>ESC</kbd>
          </div>
        </div>
        <div className="suggested-queries-wrapper">
          <p>Suggested Queries: </p>
          {isFetchingSuggestedQueries
            ? Array.from(Array(2).keys()).map((k) => (
                <button key={k} disabled className="suggested-query">
                  <LoadingIcon className="w-5 h-4" />
                </button>
              ))
            : suggestedQueries.map((q) => (
                <button
                  onClick={() => setQuery(q)}
                  key={q}
                  className="suggested-query"
                >
                  {q}
                </button>
              ))}
        </div>
      </div>

      <ul className="trieve-elements-search">
        {results.length && props.chat ? (
          <li>
            <button className="item start-chat" onClick={() => setMode("chat")}>
              <div>
                <AIIcon />
                <div>
                  <h4>
                    Can you tell me about <span>{query}</span>
                  </h4>
                  <p className="description">Use AI to answer your question</p>
                </div>
              </div>
              <ArrowIcon />
            </button>
          </li>
        ) : null}
        {results.map((result, index) => (
          <Item item={result} index={index} key={result.chunk.id} />
        ))}
      </ul>
    </>
  );
};
