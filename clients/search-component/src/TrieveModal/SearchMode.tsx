import React from "react";
import { Item } from "./item";
import { AIIcon, ArrowIcon, ReloadIcon } from "./icons";
import { useSuggestedQueries } from "../utils/hooks/useSuggestedQueries";
import { useModalState } from "../utils/hooks/modal-context";

export const SearchMode = () => {
  const { props, results, loadingResults, query, setQuery, requestID, inputRef, setMode } =
    useModalState();
  const {
    suggestedQueries,
    refetchSuggestedQueries,
    isLoadingSuggestedQueries,
  } = useSuggestedQueries();

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
        {props.suggestedQueries && (!query || (query && !results.length)) && (
          <div className="suggested-queries-wrapper">
            <>
              <button
                onClick={refetchSuggestedQueries}
                disabled={isLoadingSuggestedQueries}
                className="suggested-query"
                title="Refresh suggested queries"
              >
                <ReloadIcon width="14" height="14" />
              </button>
              <p>Suggested Queries: </p>
              {!suggestedQueries.length && (
                <p className="suggested-query empty-state-loading">
                  Loading random query suggestions...
                </p>
              )}
              {suggestedQueries.map((q) => (
                <button
                  onClick={() => setQuery(q)}
                  key={q}
                  className={`suggested-query${
                    isLoadingSuggestedQueries ? " loading" : ""
                  }`}
                >
                  {q}
                </button>
              ))}
            </>
          </div>
        )}
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
        {results.length
          ? results.map((result, index) => (
              <Item item={result} index={index} requestID={requestID} key={result.chunk.id} />
            ))
          : null}
        {query && !results.length && !loadingResults ? (
          <p className="no-results">No results found</p>
        ) : query && !results.length && loadingResults ? (
          <p className="no-results-loading">Searching...</p>
        ) : null}
      </ul>
    </>
  );
};
