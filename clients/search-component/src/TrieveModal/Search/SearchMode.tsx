import React, { Suspense } from "react";
import { Item } from "./item";
import { AIIcon, ArrowIcon, ReloadIcon } from "../icons";
import { useSuggestedQueries } from "../../utils/hooks/useSuggestedQueries";
import { useModalState } from "../../utils/hooks/modal-context";
import { Tags } from "./Tags";
import { useChatState } from "../../utils/hooks/chat-context";

export const SearchMode = () => {
  const {
    props,
    results,
    loadingResults,
    query,
    setQuery,
    requestID,
    inputRef,
    open,
    mode,
  } = useModalState();
  const {
    suggestedQueries,
    refetchSuggestedQueries,
    isLoadingSuggestedQueries,
  } = useSuggestedQueries();
  const { switchToChatAndAskQuestion } = useChatState();

  React.useEffect(() => {
    if (mode == "search" && open) {
      inputRef.current?.focus();
    }
  }, [mode, open]);

  return (
    <Suspense fallback={<div className="hidden"> </div>}>
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

          <button className="clear-query" onClick={() => setQuery("")}>
            <svg
              className="clear-query-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
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
            <button
              className="item start-chat"
              onClick={() => switchToChatAndAskQuestion(query)}
            >
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
              <Item
                item={result}
                index={index}
                requestID={requestID}
                key={result.chunk.id}
              />
            ))
          : null}
        {query && !results.length && !loadingResults ? (
          <div className="no-results">
            <p className="no-results-text">No results found</p>
            {props.problemLink && (
              <p>
                Believe this query should return results?{" "}
                <a
                  className="no-results-help-link"
                  href={`${props.problemLink}No results found for query: ${query} on ${props.brandName}`}
                  target="_blank"
                >
                  Contact us
                </a>
              </p>
            )}
          </div>
        ) : query && !results.length && loadingResults ? (
          <p className="no-results-loading">Searching...</p>
        ) : null}
      </ul>
      <div className={`trieve-footer search`}>
        <div className="bottom-row">
          <Tags />
          <span className="spacer" />
          <a
            className="trieve-powered"
            href="https://trieve.ai"
            target="_blank"
          >
            <img src="https://cdn.trieve.ai/trieve-logo.png" alt="logo" />
            Powered by Trieve
          </a>
        </div>
      </div>
    </Suspense>
  );
};

export default SearchMode;
