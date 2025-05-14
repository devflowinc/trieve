import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { UploadAudio } from "./UploadAudio";
import { UploadImage } from "./UploadImage";
import ImagePreview from "../ImagePreview";
import { SuggestedQueries } from "./SuggestedQueries";

export const SearchInput = () => {
  const {
    props,
    results,
    query,
    setQuery,
    setLoadingResults,
    inputRef,
    mode,
    uploadingImage,
    imageUrl,
    audioBase64,
    isRecording,
  } = useModalState();

  return (
    <div className={`input-wrapper ${props.type} tv-pt-2 trieve-mode-${mode}`}>
      <div className="input-flex group-focus:tv-border has-[:focus]:tv-border has-[:focus]:tv-border-[var(--tv-prop-brand-color)] sm:tv-text-sm sm:tv-leading-6 tv-py-1.5 tv-px-4 tv-items-center tv-flex tv-justify-between tv-w-full tv-rounded-lg tv-border-[1px] tv-mb-2">
        <input
          ref={inputRef}
          value={audioBase64 && query.length == 0 ? "Searching..." : query}
          onChange={(e) => {
            setLoadingResults(true);
            setQuery(e.target.value);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && props.searchBar) {
              window.location.href = "/search?q=" + query;
            }
          }}
          placeholder={
            imageUrl.length > 0
              ? "Using Image for Search"
              : isRecording
                ? "Recording... Press stop icon to submit"
                : props.placeholder || "Search for anything"
          }
          className={`search-input focus:tv-ring-0 tv-ring-0 tv-grow tv-py-1.5 tv-pr-8 ${props.type} tv-outline-none tv-border-none`}
          disabled={imageUrl.length > 0}
        />
        <div className="right-side tv-items-center tv-flex tv-gap-2.5 tv-text-base">
          <UploadAudio />
          <UploadImage />
          {query ? (
            <button onClick={() => setQuery("")}>
              <svg
                className="clear-query-icon tv-w-[16px] tv-h-[16px]  tv-fill-current"
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
              <i className="fa-solid fa-magnifying-glass tv-fill-current" />
            </div>
          )}
        </div>
      </div>
      <ImagePreview isUploading={uploadingImage} imageUrl={imageUrl} active />
      {props.suggestedQueries && (!query || (query && !results.length)) && (
        <SuggestedQueries />
      )}
    </div>
  );
};
