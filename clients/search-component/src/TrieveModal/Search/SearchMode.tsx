import React, { useEffect, useMemo } from "react";
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
import { cn } from "../../utils/styles";
import { SearchInput } from "./SearchInput";
import { GoToChatPrompt } from "./GoToChatPrompt";
import { FilterSidebar } from "../FilterSidebarComponents";

export const SearchMode = () => {
  const {
    props,
    results,
    loadingResults,
    query,
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
<>
      {props.type === "ecommerce" && props.inline && props.defaultSearchMode === "search" && (
        <div className="tv-grid tv-grid-cols-1">
          <p className="tv-text-[24px] tv-text-center tv-my-8 tv-flex tv-flex-col tv-gap-2 tv-col-span-full">
            Search Results
          </p>
        </div>
      )}
      <SearchInput />
        <div className="tv-flex tv-flex-grow">
          <SearchPage />
          <ul
            className={cn(
              `trieve-elements-${props.type} tv-grow`,
              "tv-max-w-[70vw]",
              props.type === "ecommerce" && !props.inline  &&
                "tv-grid tv-grid-cols-2 sm:tv-grid-cols-3 md:tv-grid-cols-4 lg:tv-grid-cols-5 tv-gap-2 tv-mt-0.5 tv-py-2 tv-pr-0.5",
              props.type === "ecommerce" && props.inline && props.defaultSearchMode === "search" &&             
                "tv-grid tv-grid-cols-2 sm:tv-grid-cols-3 md:tv-grid-cols-4 tv-gap-4 tv-mt-0.5 tv-py-2 tv-pr-0.5 ",
              props.type === "ecommerce" && props.inline && props.defaultSearchMode !== "search" && "tv-grid tv-grid-cols-1",
              "tv-overflow-y-auto",
            )}
          >
            {resultsLength && props.chat && imageUrl.length == 0 && props.defaultSearchMode !== "search" ? (
              <GoToChatPrompt />
            ) : null}
            {props.type === "pdf" ? (
              <div className="tv-grid md:tv-grid-cols-3">{resultsDisplay}</div>
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
        </div>
    </>
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

const SearchPage = () => {
  const { props } = useModalState();
  if (!props.searchPageProps?.display) return null;
  console.log(props.searchPageProps?.filterSidebarProps?.sections);
  return (
    <div
      className="trieve-search-page"
      data-display={props.searchPageProps?.display ? "true" : "false"}
    >
      <div className="trieve-search-page-main-section">
        <div className="trieve-filter-main-section">
          <FilterSidebar
            sections={
              props.searchPageProps?.filterSidebarProps?.sections ?? []
            }
          />
        </div>
      </div>
    </div>
  );
};

export default SearchMode;
