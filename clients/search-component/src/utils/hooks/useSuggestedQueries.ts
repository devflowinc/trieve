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

  const getQueries = async (abortController: AbortController) => {
    if (isFetching.current) {
      return;
    }
    isFetching.current = true;
    setIsLoading(true);
    const queries = await getSuggestedQueries({
      trieve: trieveSDK,
      query,
      count: props.numberOfSuggestions ?? 3,
      abortController,
    });
    setSuggestedQueries(queries.queries);
    isFetching.current = false;
    setIsLoading(false);
  };

  useEffect(() => {
    const defaultQueries =
      props.defaultSearchQueries?.filter((q) => q !== "") ?? [];

    if (defaultQueries.length) {
      setSuggestedQueries(defaultQueries);
      return;
    }

    const abortController = new AbortController();
    const timeoutId = setTimeout(async () => {
      await getQueries(abortController);
    }, 1);

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("Component unmounted");
    };
  }, []);

  return {
    suggestedQueries,
    getQueries,
    isLoadingSuggestedQueries: isLoading,
  };
};
