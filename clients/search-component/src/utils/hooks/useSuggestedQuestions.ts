import { useEffect, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQuestions } from "../trieve";
import { useModalState } from "./modal-context";

export const useSuggestedQuestions = () => {
  const { props, query, trieveSDK, currentGroup } = useModalState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQuestions, setSuggestedQuestions] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getQuestions = async () => {
    setIsLoading(true);
    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
      count: props.numberOfSuggestions ?? 3,
      groupTrackingId: props.inline
        ? (props.groupTrackingId ?? currentGroup?.tracking_id)
        : currentGroup?.tracking_id,
      query,
      props,
    });
    setSuggestedQuestions(
      queries.queries.map((q) => {
        return q.replace(/^[\d.-]+\s*/, "").trim();
      })
    );
    setIsLoading(false);
  };

  useEffect(() => {
    if (props.defaultAiQuestions?.length) {
      setSuggestedQuestions(props.defaultAiQuestions);
      return;
    }

    setIsLoading(true);
    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      const queries = await getSuggestedQuestions({
        trieve: trieveSDK,
        count: props.numberOfSuggestions ?? 3,
        abortController,
        query,
        groupTrackingId: props.inline
          ? (props.groupTrackingId ?? currentGroup?.tracking_id)
          : currentGroup?.tracking_id,
        props,
      });
      setSuggestedQuestions(
        queries.queries.map((q) => {
          return q.replace(/^[\d.-]+\s*/, "").trim();
        })
      );
      setIsLoading(false);
    });

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("fetch aborted");
    };
  }, []);

  return {
    suggestedQuestions,
    getQuestions,
    isLoadingSuggestedQueries: isLoading,
  };
};
