import { throttle } from "lodash";
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

  const getQueries = throttle(async () => {
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
  }, 1000);

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

    getQueries();
  }, [query]);

  return {
    suggestedQueries,
    refetchSuggestedQueries,
    isLoadingSuggestedQueries: isLoading,
  };
};
