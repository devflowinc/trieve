import React, { useCallback, useEffect, useState } from "react";
import { cn } from "../../utils/styles";
import { useModalState } from "../../utils/hooks/modal-context";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQueries } from "../../utils/trieve";

export const SuggestedQueries = () => {
  const { props, query, setQuery, imageUrl, trieveSDK } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQueries, setSuggestedQueries] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  console.log(props.suggestedQueries && query);

  const getQueries = useCallback(
    (abortController: AbortController) => {
      setIsLoading(true);
      getSuggestedQueries({
        trieve: trieveSDK,
        query,
        count: props.numberOfSuggestions ?? 3,
        abortController,
      }).then((suggestedQueriesResp) => {
        setSuggestedQueries(suggestedQueriesResp.queries);
        setIsLoading(false);
      });
    },
    [query],
  );

  useEffect(() => {
    const defaultQueries =
      props.defaultSearchQueries?.filter((q) => q !== "") ?? [];

    if (defaultQueries.length) {
      setSuggestedQueries(defaultQueries);
      return;
    }

    const abortController = new AbortController();
    getQueries(abortController);

    return () => {
      abortController.abort("Component unmounted");
      setIsLoading(false);
    };
  }, []);

  return (
    <div
      className={cn(
        `suggested-queries-wrapper tv-flex tv-mt-2 tv-gap-2 tv-items-center tv-flex-wrap tv-mb-2 ${props.type}`,
        imageUrl && "tv-pt-2",
      )}
    >
      <button
        onClick={() => getQueries(new AbortController())}
        disabled={isLoading}
        className="suggested-query"
        title="Refresh suggested queries"
      >
        <i className="fa-solid fa-arrow-rotate-right"></i>
      </button>
      {suggestedQueries.length === 0 ? (
        <div className="suggested-query loading">Loading...</div>
      ) : (
        suggestedQueries.map((q) => {
          q = q.replace(/^-|\*$/g, "");
          q = q.trim();
          return (
            <button
              onClick={() => setQuery(q)}
              key={q}
              className={`suggested-query${isLoading ? " loading" : ""}`}
            >
              {q}
            </button>
          );
        })
      )}
    </div>
  );
};
