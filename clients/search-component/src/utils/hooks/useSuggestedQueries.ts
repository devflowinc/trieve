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
      abortController,
    });
    setSuggestedQueries(queries.queries.splice(0, 3));
    isFetching.current = false;
    setIsLoading(false);
  };

  useEffect(() => {
    const abortController = new AbortController();

    const defaultQueries =
      props.defaultSearchQueries?.filter((q) => q !== "") ?? [];

    if (!defaultQueries || !defaultQueries.length) {
      getQueries(abortController);
      return;
    }
    setSuggestedQueries(defaultQueries);

    return () => {
      abortController.abort();
    };
  }, []);

  useEffect(() => {
    if (!props.suggestedQueries || query === "") {
      return;
    }

    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      getQueries(abortController);
    }, props.debounceMs);

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("Query changed");
    };
  }, [query]);

  return {
    suggestedQueries,
    getQueries,
    isLoadingSuggestedQueries: isLoading,
  };
};
