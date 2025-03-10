import React from "react";
import { cn } from "../../utils/styles";
import { useSuggestedQueries } from "../../utils/hooks/useSuggestedQueries";
import { useModalState } from "../../utils/hooks/modal-context";

export const SuggestedQueries = () => {
  const { suggestedQueries, getQueries, isLoadingSuggestedQueries } =
    useSuggestedQueries();

  const { props, setQuery, imageUrl } = useModalState();

  return (
    <div
      className={cn(
        `suggested-queries-wrapper tv-flex tv-mt-2 tv-gap-2 tv-items-center tv-flex-wrap tv-mb-2 ${props.type}`,
        imageUrl && "tv-pt-2",
      )}
    >
      <button
        onClick={() => getQueries(new AbortController())}
        disabled={isLoadingSuggestedQueries}
        className="suggested-query"
        title="Refresh suggested queries"
      >
        <i className="fa-solid fa-arrow-rotate-right"></i>
      </button>
      {suggestedQueries.length === 0 && (
        <div className="suggested-query loading">Loading...</div>
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
  );
};
