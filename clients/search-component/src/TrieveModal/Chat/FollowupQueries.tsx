import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useFollowupQuestions } from "../../utils/hooks/useFollowupQuestions";
import { SparklesIcon } from "../icons";
import { useAutoAnimate } from "@formkit/auto-animate/react";
import { useModalState } from "../../utils/hooks/modal-context";

export const FollowupQueries = () => {
  const { props, trieveSDK, fingerprint, abTreatment } = useModalState();
  const { isDoneReading, askQuestion, messages } = useChatState();
  const { suggestedQuestions, isLoadingSuggestedQueries } =
    useFollowupQuestions();
  const [parent] = useAutoAnimate();

  if (!isDoneReading || props.previewTopicId) {
    return null;
  }

  const handleFollowupQuery = async (q: string) => {
    askQuestion(q);

    const requestId =
      messages[messages.length - 1]?.queryId ??
      "00000000-0000-0000-0000-000000000000";

    await trieveSDK.sendAnalyticsEvent({
      event_name: `site-followup_query`,
      event_type: "click",
      user_id: fingerprint,
      location: window.location.href,
      metadata: {
        followup_query: q,
        component_props: props,
        ab_treatment: abTreatment,
      },
      request: {
        request_id: requestId,
        request_type: "rag",
      },
    });
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
