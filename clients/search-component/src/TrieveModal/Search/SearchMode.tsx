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
import { UploadAudio } from "./UploadAudio";
import { ModeSwitch } from "../ModeSwitch";

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
    audioBase64,
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
    <Suspense fallback={<div className="suspense-fallback"></div>}>
      {!props.inline && (
        <div
          className={`mode-switch-wrapper tv-flex tv-items-center tv-px-2 tv-gap-2 tv-justify-end tv-mt-2 tv-font-medium ${mode}`}
        >
          <ModeSwitch />
          <div
            className={`tv-text-xs tv-rounded-md !tv-bg-transparent tv-flex !hover:bg-tv-zinc-200 tv-px-2 tv-justify-end tv-items-center tv-p-2 tv-gap-0.5 tv-cursor-pointer ${props.type}`}
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
        </div>
      )}
      <div className={`input-wrapper ${props.type} ${mode}`}>
        <div className="input-flex group-focus:tv-border has-[:focus]:tv-border has-[:focus]:tv-border-[var(--tv-prop-brand-color)] tv-mb-2 sm:tv-text-sm sm:tv-leading-6 tv-px-4 tv-items-center tv-flex tv-justify-between tv-w-full tv-rounded-lg tv-border-[1px]">
          <input
            ref={inputRef}
            value={audioBase64 && query.length == 0 ? "Searching..." : query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={
              imageUrl.length > 0
                ? "Using Image for Search"
                : props.placeholder || "Search for anything"
            }
            className={`search-input focus:tv-ring-0 tv-ring-0 tv-grow tv-py-1.5 tv-pr-8 ${props.type} tv-outline-none tv-border-none`}
            disabled={imageUrl.length > 0}
          />
          <div className="right-side tv-items-center flex gap-2">
            <UploadAudio />
            <UploadImage />
            {query ? (
              <button onClick={() => setQuery("")}>
                <svg
                  className="clear-query-icon tv-w-[14px] tv-h-[14px] tv-fill-current"
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
            ) : (
              <div>
                <i className="fa-solid fa-magnifying-glass" />
              </div>
            )}
          </div>
        </div>
        <ImagePreview isUploading={uploadingImage} imageUrl={imageUrl} active />
        {props.suggestedQueries && (!query || (query && !results.length)) && (
          <div className={`suggested-queries-wrapper ${props.type}`}>
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

        {(imageUrl || query || audioBase64) &&
        !resultsLength &&
        !loadingResults ? (
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
        ) : (imageUrl || query || audioBase64) &&
          !resultsLength &&
          loadingResults ? (
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
