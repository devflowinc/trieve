import React, { Suspense } from "react";
import { DocsItem } from "./DocsItem";
import { AIIcon, ArrowIcon, ReloadIcon } from "../icons";
import { useSuggestedQueries } from "../../utils/hooks/useSuggestedQueries";
import { useModalState } from "../../utils/hooks/modal-context";
import { Tags } from "../Tags";
import { useChatState } from "../../utils/hooks/chat-context";
import {
  ChunkWithHighlights,
  GroupChunk,
  isChunksWithHighlights,
} from "../../utils/types";
import { ProductItem } from "./ProductItem";
import { ProductGroupItem } from "./ProductGroupItem";

export const SearchMode = () => {
  const {
    props,
    results,
    loadingResults,
    query,
    setQuery,
    setOpen,
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

  const getItemComponent = (
    result: ChunkWithHighlights | GroupChunk[],
    index: number,
  ) => {
    const chunkOrGroup = isChunksWithHighlights(result);
    const ecommerce = props.type == "ecommerce";
    if (chunkOrGroup && ecommerce) {
      return (
        <ProductItem
          item={result}
          index={index}
          requestID={requestID}
          key={result.chunk.id}
        />
      );
    } else if (!chunkOrGroup && ecommerce) {
      return (
        <ProductGroupItem group={result} index={index} requestID={requestID} />
      );
    } else if (chunkOrGroup) {
      return (
        <DocsItem
          item={result}
          index={index}
          requestID={requestID}
          key={result.chunk.id}
        />
      );
    } else {
      return (
        <div key={index} className="item-group-container">
          <p className="item-group-name">{result[0].group.name}</p>
          {result[0].chunks.map((chunk, index) => (
            <DocsItem
              item={chunk}
              index={index}
              requestID={requestID}
              key={chunk.chunk.id}
              className="item group"
            />
          ))}
        </div>
      );
    }
  };

  React.useEffect(() => {
    if (mode == "search" && open) {
      inputRef.current?.focus();
    }
  }, [mode, open]);

  return (
    <Suspense fallback={<div className="hidden"> </div>}>
      <div
        className={`close-modal-button search ${props.type}`}
        onClick={() => setOpen(false)}
      >
        <svg
          className="close-icon"
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
        <span>Close</span>
      </div>
      <div className={`input-wrapper ${props.type}`}>
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
            className={`search-input ${props.type}`}
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
          <div className={`suggested-queries-wrapper ${props.type}`}>
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

      <ul className={`trieve-elements-${props.type}`}>
        {results.length && props.chat ? (
          <li className="start-chat-li" key="chat">
            <button
              id="trieve-search-item-0"
              className="item start-chat"
              onClick={() => switchToChatAndAskQuestion(query)}
            >
              <div>
                <AIIcon />
                <div>
                  <h4>
                    {props.type == "docs"
                      ? "Can you tell me about "
                      : "Can you help me find "}
                    <span>{query}</span>
                  </h4>
                  <p className="description">Use AI to discover items</p>
                </div>
              </div>
              <ArrowIcon />
            </button>
          </li>
        ) : null}
        {results.length
          ? results.map((result, index) => getItemComponent(result, index))
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
          <p className={`no-results-loading ${props.type}`}>Searching...</p>
        ) : null}
      </ul>
      <div className={`trieve-footer search ${props.type}`}>
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
