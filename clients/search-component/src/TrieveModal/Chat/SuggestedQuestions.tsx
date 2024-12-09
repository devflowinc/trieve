import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
export const SuggestedQuestions = () => {
  const { askQuestion, setCurrentQuestion } = useChatState();
  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
    refetchSuggestedQuestion,
  } = useSuggestedQuestions();
  return (
    <>
      <p></p>
      <div>
        <p className="header">
          <button
            onClick={refetchSuggestedQuestion}
            disabled={isLoadingSuggestedQueries}
            className="suggested-question refetch"
            title="Refresh suggested questions"
          >
            <i className="fa-solid fa-arrow-rotate-right"></i>
          </button>{" "}
          Example questions
        </p>
        <div className="questions">
          {!suggestedQuestions.length && (
            <p className="suggested-question empty-state-loading">
              Loading example questions...
            </p>
          )}
          {suggestedQuestions.length ? (
            <>
              {suggestedQuestions.map((q) => (
                <button
                  onClick={() => {
                    setCurrentQuestion(q);
                    askQuestion(q);
                  }}
                  key={q}
                  className={`suggested-question ${
                    isLoadingSuggestedQueries ? "loading" : ""
                  }`}
                >
                  {q}
                </button>
              ))}
            </>
          ) : null}
        </div>
      </div>
    </>
  );
};
