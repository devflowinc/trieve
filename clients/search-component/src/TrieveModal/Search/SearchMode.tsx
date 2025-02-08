import React, { Suspense, useEffect, useMemo } from "react";
import { DocsItem } from "./DocsItem";
import { ModalProps, useModalState } from "../../utils/hooks/modal-context";
import {
  ChunkWithHighlights,
  GroupChunk,
  isChunkWithHighlights,
  isPdfChunk,
} from "../../utils/types";
import { ProductItem } from "./ProductItem";
import { ProductGroupItem } from "./ProductGroupItem";
import { PdfItem } from "./PdfItem";
import { ModeSwitch } from "../ModeSwitch";
import { cn } from "../../utils/styles";
import { SearchInput } from "./SearchInput";
import { GoToChatPrompt } from "./GoToChatPrompt";

export const SearchMode = () => {
  const {
    props,
    results,
    loadingResults,
    query,
    setOpen,
    requestID,
    inputRef,
    open,
    mode,
    imageUrl,
    audioBase64,
  } = useModalState();

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

  const hasQuery = imageUrl || query || audioBase64;

  return (
    <Suspense fallback={<div className="tv-hidden"></div>}>
      {!props.inline && (
        <div className="mode-switch-wrapper tv-flex tv-items-center tv-px-2 tv-gap-2 tv-justify-end tv-mt-2 tv-font-medium ${mode}">
          <ModeSwitch />
          <div
            className={`tv-text-xs tv-rounded-md !tv-bg-transparent tv-flex !hover:bg-tv-zinc-200 tv-px-2 tv-justify-end tv-items-center tv-p-2 tv-gap-0.5 tv-cursor-pointer ${props.type}`}
            onClick={() => setOpen(false)}
          >
            <CloseIcon />
          </div>
        </div>
      )}
      <SearchInput />
      {resultsLength && props.chat && imageUrl.length == 0 ? (
        <GoToChatPrompt />
      ) : null}
      <ul
        className={cn(
          `trieve-elements-${props.type} tv-grow`,
          props.type === "ecommerce" &&
            "tv-grid tv-grid-cols-2 sm:tv-grid-cols-3 md:tv-grid-cols-4 lg:tv-grid-cols-5 tv-gap-2 tv-mt-0.5 tv-py-2 tv-max-w-7xl tv-mx-auto tv-pr-0.5",
        )}
      >
        {props.type === "pdf" ? (
          <div className="tv-grid tv-grid-cols-2">{resultsDisplay}</div>
        ) : (
          resultsDisplay
        )}
      </ul>

      {hasQuery && !resultsLength && !loadingResults && (
        <NoResults props={props} query={query} />
      )}
      {hasQuery && !resultsLength && loadingResults && (
        <div className="tv-text-sm tv-animate-pulse tv-text-center tv-my-8 tv-flex tv-flex-col tv-gap-2 tv-col-span-full">
          <p className="">Searching...</p>
        </div>
      )}
    </Suspense>
  );
};

const NoResults = ({ props, query }: { props: ModalProps; query: string }) => {
  return (
    <div className="tv-text-sm tv-text-center tv-my-8 tv-flex tv-flex-col tv-gap-2 tv-col-span-full">
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
  );
};

const CloseIcon = () => {
  return (
    <>
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
    </>
  );
};

export default SearchMode;
