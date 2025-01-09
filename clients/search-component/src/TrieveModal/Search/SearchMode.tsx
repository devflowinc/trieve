import React, { Suspense, useEffect, useMemo } from "react";
import { DocsItem } from "./DocsItem";
import { useSuggestedQueries } from "../../utils/hooks/useSuggestedQueries";
import { useModalState } from "../../utils/hooks/modal-context";
import { Tags } from "../Tags";
import { useChatState } from "../../utils/hooks/chat-context";
import {
  ChunkWithHighlights,
  GroupChunk,
  isChunkWithHighlights,
  isPdfChunk,
} from "../../utils/types";
import { ProductItem } from "./ProductItem";
import { ProductGroupItem } from "./ProductGroupItem";
import { PdfItem } from "./PdfItem";
import { SparklesIcon } from "../icons";
import { cn } from "../../utils/styles";

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

  const { suggestedQueries, getQueries, isLoadingSuggestedQueries } =
    useSuggestedQueries();

  const { switchToChatAndAskQuestion } = useChatState();

  const getItemComponent = (
    result: ChunkWithHighlights | GroupChunk[],
    index: number,
  ) => {
    const isChunk = isChunkWithHighlights(result);

    // Target non group pdf search
    if (isChunk && props.type === "pdf") {
      if (isPdfChunk(result)) {
        return (
          <PdfItem
            item={result}
            index={index}
            requestID={requestID}
            key={result.chunk.id}
          />
        );
      }
    }

    if (isChunk && props.type === "ecommerce") {
      return (
        <ProductItem
          item={result}
          index={index}
          requestID={requestID}
          key={result.chunk.id}
        />
      );
    } else if (!isChunk && props.type == "ecommerce") {
      return (
        <ProductGroupItem
          key={result[0].group.id}
          group={result}
          index={index}
          requestID={requestID}
        />
      );
    } else if (isChunk) {
      return (
        <DocsItem
          key={result.chunk.id}
          item={result}
          index={index}
          requestID={requestID}
        />
      );
    } else {
      return (
        <div key={index} className="item-group-container">
          <p className="item-group-name">{result[0].group.name}</p>
          {result[0].chunks.map((chunk, index) => (
            <DocsItem
              key={chunk.chunk.id}
              item={chunk}
              index={index}
              requestID={requestID}
              className="item group"
            />
          ))}
        </div>
      );
    }
  };

  useEffect(() => {
    if (mode == "search" && open) {
      inputRef.current?.focus();
    }
  }, [mode, open]);

  const resultsLength = useMemo(() => results.length, [results]);

  const resultsDisplay = useMemo(() => {
    if (results.length) {
      const comps = results.map((result, index) =>
        getItemComponent(result, index),
      );
      return comps;
    } else {
      return null;
    }
  }, [results]);

  return (
    <Suspense fallback={<div className="suspense-fallback"> </div>}>
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
      <div
        className={cn(
          `input-wrapper sticky top-0 z-10 flex flex-col gap-2 rounded-lg ${props.type}`,
          props.type === "ecommerce" && "max-w-7xl mx-auto",
        )}
      >
        <div className="input-flex flex items-center rounded-lg">
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
            className={cn(
              "search-input pr-8",
              props.type === "ecommerce" && "rounded-lg",
            )}
          />

          <button
            className="clear-query flex items-center justify-end mt-2 absolute top-1.5 right-2 z-30 font-medium"
            onClick={() => setQuery("")}
          >
            <svg
              className="clear-query-icon w-5 h-5 fill-current"
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
          <div
            className={cn(
              "suggested-queries-wrapper flex gap-2 items-center flex-wrap mb-2",
              props.type === "ecommerce" && "pl-0",
            )}
          >
            <>
              <button
                onClick={() => getQueries(new AbortController())}
                disabled={isLoadingSuggestedQueries}
                className="suggested-query inline-flex items-center rounded-md px-2 py-1 text-xs text-left"
                title="Refresh suggested queries"
              >
                <i className="fa-solid fa-arrow-rotate-right"></i>
              </button>
              <p>Suggested Queries: </p>
              {!suggestedQueries.length && (
                <p className="suggested-query inline-flex items-center rounded-md px-2 py-1 text-xs text-left empty-state-loading">
                  Loading random query suggestions...
                </p>
              )}
              {suggestedQueries.map((q) => {
                q = q.replace(/^-|\*$/g, "");
                q = q.trim();
                return (
                  <button
                    onClick={() => setQuery(q)}
                    key={q}
                    className={`suggested-query inline-flex items-center rounded-md px-2 py-1 text-xs text-left empty-state-loading ${
                      isLoadingSuggestedQueries ? " loading" : ""
                    }`}
                  >
                    {q}
                  </button>
                );
              })}
            </>
          </div>
        )}
      </div>

      <ul className={`trieve-elements-${props.type}`}>
        {resultsLength && props.chat ? (
          <li className="start-chat-li" key="chat">
            <button
              id="trieve-search-item-0"
              className="item start-chat"
              onClick={() => switchToChatAndAskQuestion(query)}
            >
              <div
                style={{
                  paddingLeft: props.type === "ecommerce" ? "1rem" : "",
                }}
              >
                <SparklesIcon />
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
              <i className="fa-solid fa-chevron-right"></i>
            </button>
          </li>
        ) : null}

        {props.type === "pdf" ? (
          <div className="pdf-results">{resultsDisplay}</div>
        ) : (
          resultsDisplay
        )}

        {query && !resultsLength && !loadingResults ? (
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
        ) : query && !resultsLength && loadingResults ? (
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
