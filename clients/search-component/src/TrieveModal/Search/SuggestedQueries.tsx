import React, { useCallback, useEffect, useState } from "react";
import { cn } from "../../utils/styles";
import { useModalState, isDefaultSearchQuery } from "../../utils/hooks/modal-context";
import { DefaultSearchQuery } from "trieve-ts-sdk";
import { getSuggestedQueries } from "../../utils/trieve";

export const SuggestedQueries = () => {
  const { props, query, setQuery, imageUrl, trieveSDK } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQueries, setSuggestedQueries] = useState<
    (DefaultSearchQuery | string)[]
  >([]);

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

  const handleSendSuggestedQuery = (q: DefaultSearchQuery | string) => {
    if (isDefaultSearchQuery(q)) {
      setQuery(q.query ?? "");

      if (q.imageUrl) {
        setImageUrl(q.imageUrl);
      }
    } else {
      setQuery(q);
    }
  };

  useEffect(() => {
    const defaultQueries =
      props.defaultSearchQueries && props.defaultSearchQueries.length > 0 && isDefaultSearchQuery(props.defaultSearchQueries[0])
        ? props.defaultSearchQueries
        : props.defaultSearchQueries?.filter((q) => q !== "") ?? [];

    if (props.defaultSearchQueries?.length) {
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
          let query = isDefaultSearchQuery(q)  ? q.query?.replace(/^-|\*$/g, "") : q;
          query = query?.trim();
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
