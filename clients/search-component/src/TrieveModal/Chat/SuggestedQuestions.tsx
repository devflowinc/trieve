import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import {
  AiQuestion,
  isAiQuestion,
  isDefaultSearchQuery,
  useModalState,
} from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";
import { useAutoAnimate } from "@formkit/auto-animate/react";
import { ArrowRotateRightIcon, SparklesIcon } from "../icons";
import { AIInitialMessage } from "./AIInitalMessage";
import { DefaultSearchQuery } from "trieve-ts-sdk";

export const SuggestedQuestions = ({
  onMessageSend,
}: {
  onMessageSend?: () => void;
}) => {
  const { askQuestion, messages, setCurrentQuestion } = useChatState();
  const { suggestedQuestions, isLoadingSuggestedQueries, getQuestions } =
    useSuggestedQuestions();

  const { props, trieveSDK, fingerprint, abTreatment } = useModalState();
  const [parent] = useAutoAnimate({ duration: 100 });

  if (messages.length) {
    return null;
  }

  const handleSuggestedQuestion = async (
    q: AiQuestion | DefaultSearchQuery | string,
  ) => {
    setCurrentQuestion(
      isAiQuestion(q)
        ? q.questionText
        : isDefaultSearchQuery(q)
          ? (q.query ?? "")
          : q,
    );

    if (!isDefaultSearchQuery(q)) {
      askQuestion(
        isAiQuestion(q) ? q.questionText : q,
        undefined,
        isAiQuestion(q) ? (q.products?.map((p) => p.groupId) ?? []) : undefined,
        isAiQuestion(q) && q.promptForAI !== "" ? q.promptForAI : undefined,
      );
    } else {
      askQuestion(
        isAiQuestion(q)
          ? q.questionText
          : isDefaultSearchQuery(q)
            ? (q.query ?? "")
            : q,
        undefined,
        isAiQuestion(q) ? (q.products?.map((p) => p.groupId) ?? []) : undefined,
        isAiQuestion(q) && q.promptForAI !== "" ? q.promptForAI : undefined,
        undefined,
        q.imageUrl ?? "",
      );
    }

    const requestId =
      messages[messages.length - 1]?.queryId ??
      "00000000-0000-0000-0000-000000000000";

    await trieveSDK.sendAnalyticsEvent({
      event_name: `site-followup_query`,
      event_type: "click",
      user_id: fingerprint,
      location: window.location.href,
      metadata: {
        followup_query: isAiQuestion(q)
          ? q.questionText
          : isDefaultSearchQuery(q)
            ? (q.query ?? "")
            : q,
        component_props: props,
        ab_treatment: abTreatment,
      },
      request: {
        request_id: requestId,
        request_type: "rag",
      },
      is_conversion: false,
    });
    if (onMessageSend) {
      onMessageSend();
    }
  };

  return (
    <div className="ai-message initial-message">
      <AIInitialMessage />
      <div
        className={cn(
          props.inline && "tv-flex tv-gap-x-3 tv-flex-wrap tv-items-center",
        )}
      >
        <div
          ref={parent}
          className={cn(
            "questions tv-pt-2 ",
            props.inline && "inline-questions !tv-pb-0",
          )}
        >
          {props.allowRefreshSuggestedQueries && (
            <button
              onClick={() => getQuestions()}
              disabled={isLoadingSuggestedQueries}
              className="suggested-question tv-cursor-pointer tv-border tv-rounded-md tv-p-1 tv-text-xs disabled:tv-cursor-not-allowed tv-text-center"
              title="Refresh suggested questions"
            >
              <ArrowRotateRightIcon
                height={15}
                width={15}
                className="refresh-suggestions-icon"
              />
            </button>
          )}{" "}
          {!suggestedQuestions?.length && (
            <span className="suggested-question tv-text-nowrap empty-state-loading">
              Loading example questions...
            </span>
          )}
          {suggestedQuestions?.map((q) => (
            <button
              onClick={() => {
                handleSuggestedQuestion(q);
              }}
              key={
                isAiQuestion(q)
                  ? q.questionText
                  : isDefaultSearchQuery(q)
                    ? (q.query ?? "")
                    : q
              }
              className={`suggested-question tv-flex tv-gap-1 tv-items-center${
                isLoadingSuggestedQueries ? " loading" : ""
              }`}
            >
              <SparklesIcon fill="none" width={15} height={15} />
              {isAiQuestion(q)
                ? q.questionText
                : isDefaultSearchQuery(q)
                  ? (q.query ?? "")
                  : q}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
