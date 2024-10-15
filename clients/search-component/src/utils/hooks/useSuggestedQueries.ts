import { useEffect, useRef, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQueries } from "../trieve";
import { useModalState } from "./modal-context";

export const useSuggestedQueries = () => {
  const { props, query, trieveSDK } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const isFetching = useRef(false);
  const [suggestedQueries, setSuggestedQueries] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQueries = async () => {
    if (isFetching.current) {
      return;
    }
    isFetching.current = true;
    setIsLoading(true);
    const queries = await getSuggestedQueries({
      trieve: trieveSDK,
      query,
    });
    setSuggestedQueries(queries.queries.splice(0, 3));
    isFetching.current = false;
    setIsLoading(false);
  };

  const refetchSuggestedQueries = () => {
    getQueries();
  };

  useEffect(() => {
    if (!props.suggestedQueries) {
      return;
    }

    if (props.defaultSearchQueries?.length) {
      if (query) {
        getQueries();
        return;
      }
      setSuggestedQueries(props.defaultSearchQueries);
      return;
    }

    const abortController = new AbortController();

    if (isFetching.current) {
      return;
    }
    isFetching.current = true;
    setIsLoading(true);

    const timeoutId = setTimeout(async () => {
      const queries = await getSuggestedQueries({
        trieve: trieveSDK,
        query,
        abortController,
      });
      setSuggestedQueries(queries.queries.splice(0, 3));
      isFetching.current = false;
      setIsLoading(false);
    }, 500);

    return () => {
      clearTimeout(timeoutId);
      abortController.abort();
    };
  }, [query]);

  return {
    suggestedQueries,
    refetchSuggestedQueries,
    isLoadingSuggestedQueries: isLoading,
  };
};
