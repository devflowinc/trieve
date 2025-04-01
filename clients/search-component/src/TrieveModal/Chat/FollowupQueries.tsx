import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useFollowupQuestions } from "../../utils/hooks/useFollowupQuestions";
import { SparklesIcon } from "../icons";
import { useAutoAnimate } from "@formkit/auto-animate/react";
import { useModalState } from "../../utils/hooks/modal-context";

export const FollowupQueries = () => {
  const { props, trieveSDK, fingerprint } = useModalState();
  const { isDoneReading, askQuestion } = useChatState();
  const { suggestedQuestions, isLoadingSuggestedQueries } =
    useFollowupQuestions();
  const [parent] = useAutoAnimate();

  if (!isDoneReading || props.previewTopicId) {
    return null;
  }

  const handleFollowupQuery = async (q: string) => {
    const lastMessage = JSON.parse(
      window.localStorage.getItem("lastMessage") ?? "{}",
    );

    const requestId =
      Object.keys(lastMessage)[0] || "00000000-0000-0000-0000-000000000000";

    await trieveSDK.sendAnalyticsEvent({
      event_name: `site-followup_query`,
      event_type: "followup_query",
      followup_query: q,
      user_id: fingerprint,
      location: window.location.href,
      metadata: {
        component_props: props,
      },
      request: {
        request_id: requestId,
        request_type: "rag",
      },
    });

    askQuestion(q);
  };

  return (
    <div ref={parent} className="followup-questions">
      {suggestedQuestions?.map((q) => (
        <button
          onClick={() => handleFollowupQuery(q)}
          key={q}
          className={`followup-question ${
            isLoadingSuggestedQueries ? "loading" : ""
          }`}
        >
          <SparklesIcon className="followup-icon" />
          {q}
        </button>
      ))}
    </div>
  );
};
