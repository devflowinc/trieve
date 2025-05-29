import { useEffect, useState } from "react";
import { getSuggestedQuestions } from "../trieve";
import { ModalProps, useModalState } from "./modal-context";
import { DefaultSearchQuery } from "trieve-ts-sdk";

export const useSuggestedQuestions = () => {
  const { props, query, trieveSDK, currentGroup } = useModalState();
  const [isLoadingSuggestedQueries, setIsLoadingSuggestedQueries] =
    useState(false);

  const [suggestedQuestions, setSuggestedQuestions] = useState<
    ModalProps["defaultAiQuestions"] | DefaultSearchQuery[]
  >(props.defaultAiQuestions ?? []);

  const getQuestions = async () => {
    setIsLoadingSuggestedQueries(true);
    const queries = await getSuggestedQuestions({
      trieve: trieveSDK,
      count: props.numberOfSuggestions ?? 3,
      groupTrackingId: props.inline
        ? (props.groupTrackingId ?? currentGroup?.tracking_id)
        : currentGroup?.tracking_id,
      query,
      props,
      isEcommerce: props.type === "ecommerce",
    });
    setSuggestedQuestions(queries.queries);
    setIsLoadingSuggestedQueries(false);
  };

  useEffect(() => {
    if (props.defaultAiQuestions?.length) {
      setSuggestedQuestions(props.defaultAiQuestions);
      return;
    }

    setIsLoadingSuggestedQueries(true);
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
      setSuggestedQuestions(queries.queries);
      setIsLoadingSuggestedQueries(false);
    });

    return () => {
      clearTimeout(timeoutId);
      abortController.abort("fetch aborted");
    };
  }, []);

  return {
    suggestedQuestions,
    getQuestions,
    isLoadingSuggestedQueries,
  };
};
