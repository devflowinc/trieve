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
import { UploadImage } from "./UploadImage";
import ImagePreview from "../ImagePreview";

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
    uploadingImage,
    imageUrl,
  } = useModalState();

  const { suggestedQueries, getQueries, isLoadingSuggestedQueries } =
    useSuggestedQueries();

  const { switchToChatAndAskQuestion } = useChatState();

  const getItemComponent = (
    result: ChunkWithHighlights | GroupChunk[],
    index: number
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
        getItemComponent(result, index)
      );
      return comps;
    } else {
      return null;
    }
  }, [results]);

  return (
    <Suspense fallback={<div className="suspense-fallback w-96 h-96 bg-red-500">HIIIIII</div>}>
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
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={
              imageUrl.length > 0
                ? "Using Image for Search"
                : props.placeholder || "Search for anything"
            }
            className={`search-input ${props.type}`}
            disabled={imageUrl.length > 0}
          />
          {query && (
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
          )}
        </div>
        <div>
          <UploadImage />
        </div>
        <ImagePreview isUploading={uploadingImage} imageUrl={imageUrl} active />
        {props.suggestedQueries && (!query || (query && !results.length)) && (
          <div className={`suggested-queries-wrapper ${props.type}`}>
            <>
              <button
                onClick={() => getQueries(new AbortController())}
                disabled={isLoadingSuggestedQueries}
                className="suggested-query"
                title="Refresh suggested queries"
              >
                <i className="fa-solid fa-arrow-rotate-right"></i>
              </button>
              <p>Suggested Queries: </p>
              {!suggestedQueries.length && (
                <p className="suggested-query empty-state-loading">
                  Loading query suggestions...
                </p>
              )}
              {suggestedQueries.map((q) => {
                q = q.replace(/^-|\*$/g, "");
                q = q.trim();
                return (
                  <button
                    onClick={() => setQuery(q)}
                    key={q}
                    className={`suggested-query${
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
        {resultsLength && props.chat && imageUrl.length == 0 ? (
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

        {(imageUrl || query) && !resultsLength && !loadingResults ? (
          <div className="no-results">
            <p className="no-results-text">No results found</p>
            {props.problemLink && (
              <p>
                Believe this query should return results?{" "}
                <a
                  className="no-results-help-link"
                  href={`${props.problemLink}No results found for query: ${
                    query.length > 0 ? query : ""
                  } on ${props.brandName}`}
                  target="_blank"
                >
                  Contact us
                </a>
              </p>
            )}
          </div>
        ) : (imageUrl || query) && !resultsLength && loadingResults ? (
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
