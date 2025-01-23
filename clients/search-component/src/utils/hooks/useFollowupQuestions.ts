import { useEffect, useState } from "react";
import { SuggestedQueriesResponse } from "trieve-ts-sdk";
import { getSuggestedQuestions } from "../trieve";
import { useModalState } from "./modal-context";
import { useChatState } from "./chat-context";

export const useFollowupQuestions = () => {
  const { trieveSDK, currentGroup, props } = useModalState();
  const { messages } = useChatState();
  const [isLoading, setIsLoading] = useState(false);
  const [suggestedQuestions, setSuggestedQuestions] = useState<
    SuggestedQueriesResponse["queries"]
  >([]);

  const getFollowUpQuestions = async () => {
    setIsLoading(true);
    const prevMessage =
      messages
        .filter((msg) => {
          return msg.type == "user";
        })
        .slice(-1)[0] ?? messages.slice(-1)[0];

    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
      query: prevMessage.text,
      count: props.numberOfSuggestions ?? 3,
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
  };

  useEffect(() => {
    setIsLoading(true);
    const abortController = new AbortController();

    const timeoutId = setTimeout(async () => {
      getFollowUpQuestions();
    });

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("fetch aborted");
    };
  }, []);

  return {
    suggestedQuestions,
    getQuestions: getFollowUpQuestions,
    isLoadingSuggestedQueries: isLoading,
  };
};
