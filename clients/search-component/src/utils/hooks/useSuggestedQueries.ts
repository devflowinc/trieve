import throttle from "lodash.throttle";
import { useEffect, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQueries } from "../trieve";
import { useModalState } from "./modal-context";

export const useSuggestedQueries = () => {
  const { props, query } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQueries, setSuggestedQueries] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQueries = throttle(async () => {
    setIsLoading(true);
    const queries = await getSuggestedQueries({
      trieve: props.trieve,
      query,
    });
    setSuggestedQueries(queries.queries.splice(0, 2));
    setIsLoading(false);
  }, 1000);

  const refetchSuggestedQueries = getQueries;

  useEffect(() => {
    if (props.suggestedQueries && !isLoading) {
      if (query) {
        getQueries();
      } else {
        if (props.defaultQueries?.length) {
          setSuggestedQueries(props.defaultQueries.splice(0, 2));
        } else {
          getQueries();
        }
      }
    }
  }, [query]);

  return {
    isFetchingSuggestedQueries: isLoading,
    suggestedQueries,
    refetchSuggestedQueries,
  };
};
