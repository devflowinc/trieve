import { useEffect, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQuestions } from "../trieve";
import { useModalState } from "./modal-context";
import { useChatState } from "./chat-context";

export const useFollowupQuestions = () => {
  const { trieveSDK, currentGroup, props } = useModalState();
  const { currentQuestion } = useChatState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQuestions, setSuggestedQuestions] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQuestions = async () => {
    setIsLoading(true);
    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
      query: currentQuestion,
      count: props.numberOfSuggestions ?? 3,
      group: currentGroup
    });
    setSuggestedQuestions(queries.queries);
    setIsLoading(false);
  };

  const refetchSuggestedQuestion = () => {
    getQuestions();
  };

  useEffect(() => {
    setIsLoading(true);
    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      getQuestions();
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
