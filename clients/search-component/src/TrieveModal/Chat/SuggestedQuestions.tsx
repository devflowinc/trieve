import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";
import { useAutoAnimate } from "@formkit/auto-animate/react";
import { ArrowRotateRightIcon, SparklesIcon } from "../icons";
import { AIInitialMessage } from "./AIInitalMessage";

export const SuggestedQuestions = ({
  onMessageSend,
}: {
  onMessageSend?: () => void;
}) => {
  const { askQuestion, messages, setCurrentQuestion } = useChatState();
  const { suggestedQuestions, isLoadingSuggestedQueries, getQuestions } =
    useSuggestedQuestions();

  const { props, trieveSDK, fingerprint } = useModalState();
  const [parent] = useAutoAnimate({ duration: 100 });

  if (messages.length) {
    return null;
  }

  const handleSuggestedQuestion = async (q: string) => {
    setCurrentQuestion(q);

    const lastMessage = JSON.parse(
      window.localStorage.getItem("lastMessage") ?? "{}",
    );

    const requestId =
      Object.keys(lastMessage)[0] || "00000000-0000-0000-0000-000000000000";

    await trieveSDK.sendAnalyticsEvent({
      event_name: `site-followup_query`,
      event_type: "followup_query",
      user_id: fingerprint,
      location: window.location.href,
      metadata: {
        followup_query: q,
        component_props: props,
      },
      request: {
        request_id: requestId,
        request_type: "rag",
      },
    });

    askQuestion(q);

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
          <button
            onClick={() => getQuestions()}
            disabled={isLoadingSuggestedQueries}
            className="suggested-question tv-cursor-pointer tv-border tv-rounded-md tv-p-1 tv-text-xs disabled:tv-cursor-not-allowed tv-text-center"
            title="Refresh suggested questions"
          >
            <ArrowRotateRightIcon height={15} width={15} />
          </button>{" "}
          {!suggestedQuestions.length && (
            <span className="suggested-question tv-text-nowrap empty-state-loading">
              Loading example questions...
            </span>
          )}
          {suggestedQuestions?.map((q) => (
            <button
              onClick={() => {
                handleSuggestedQuestion(q);
              }}
              key={q}
              className={`suggested-question tv-flex tv-gap-1 tv-items-center${
                isLoadingSuggestedQueries ? " loading" : ""
              }`}
            >
              <SparklesIcon width={15} height={15} />
              {q}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
