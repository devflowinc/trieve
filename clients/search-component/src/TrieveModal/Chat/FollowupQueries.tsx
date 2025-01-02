import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useFollowupQuestions } from "../../utils/hooks/useFollowupQuestions";

export const FollowupQueries = () => {
  const { isDoneReading, askQuestion, setCurrentQuestion } = useChatState();

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
                    setCurrentQuestion(q);
                    askQuestion(q);
                  }}
                  key={q}
                  className={`followup-question ${isLoadingSuggestedQueries ? "loading" : ""
                    }`}
                >
                  <i className="fa-solid fa-wand-magic-sparkles followup-icon"></i>
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
