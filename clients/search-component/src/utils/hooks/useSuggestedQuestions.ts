import { useEffect, useRef, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQuestions } from "../trieve";
import { useModalState } from "./modal-context";

export const useSuggestedQuestions = () => {
  const { props, trieveSDK } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const isFetching = useRef(false);
  const [suggestedQuestions, setSuggestedQuestions] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQuestions = async () => {
    isFetching.current = true;
    setIsLoading(true);
    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
    });
    setSuggestedQuestions(queries.queries.splice(0, 3));
    isFetching.current = false;
    setIsLoading(false);
  };

  const refetchSuggestedQuestion = () => {
    getQuestions();
  };

  useEffect(() => {
    if (props.defaultAiQuestions?.length) {
      setSuggestedQuestions(props.defaultAiQuestions);
      return;
    }

    setIsLoading(true);
    isFetching.current = true;
    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      const queries = await getSuggestedQuestions({
        trieve: trieveSDK,
        abortController,
      });
      setSuggestedQuestions(queries.queries.splice(0, 3));
      isFetching.current = false;
      setIsLoading(false);
    });

    return () => {
      clearTimeout(timeoutId);
      abortController.abort();
    };
  }, []);

  return {
    suggestedQuestions,
    refetchSuggestedQuestion,
    isLoadingSuggestedQueries: isLoading,
  };
};
