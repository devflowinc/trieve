import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useFollowupQuestions } from "../../utils/hooks/useFollowupQuestions";
import { SparklesIcon } from "../icons";

export const FollowupQueries = () => {
  const { isDoneReading, askQuestion } = useChatState();

  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
  } = useFollowupQuestions();

  if (isDoneReading == true) {
    return (
      <div>
        <div className="followup-questions">
          {suggestedQuestions.length ? (
            <>
              {suggestedQuestions.map((q) => (
                <button
                  onClick={() => {
                    askQuestion(q);
                  }}
                  key={q}
                  className={`followup-question ${isLoadingSuggestedQueries ? "loading" : ""
                    }`}
                >
                  <SparklesIcon className="followup-icon" />
                  {q}
                </button>
              ))}
            </>
          ) : null}
        </div>
      </div>
    );
  } else {
    return null
  }
};
