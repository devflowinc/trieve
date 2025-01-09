import { useEffect, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQuestions } from "../trieve";
import { useModalState } from "./modal-context";

export const useFollowupQuestions = () => {
  const { query, trieveSDK, currentGroup, props } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQuestions, setSuggestedQuestions] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQuestions = async () => {
    setIsLoading(true);
    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
      query,
      count: props.numberOfSuggestions ?? 3,
      group: currentGroup
    });
    setSuggestedQuestions(queries.queries.splice(0, 3));
    setIsLoading(false);
  };

  const refetchSuggestedQuestion = () => {
    getQuestions();
  };

  useEffect(() => {
    setIsLoading(true);
    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      const queries = await getSuggestedQuestions({
        trieve: trieveSDK,
        abortController,
        group: currentGroup,
        query,
        count: props.numberOfSuggestions ?? 3,
        context: "You are an assistant searching through a docs website"
      });
      setSuggestedQuestions(queries.queries);
      setIsLoading(false);
    });

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("fetch aborted");
    };
  }, []);

  return {
    suggestedQuestions,
    refetchSuggestedQuestion,
    isLoadingSuggestedQueries: isLoading,
  };
};
