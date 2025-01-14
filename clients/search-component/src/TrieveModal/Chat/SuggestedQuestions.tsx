import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";

export const SuggestedQuestions = () => {
  const { askQuestion, setCurrentQuestion } = useChatState();
  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
    refetchSuggestedQuestion,
  } = useSuggestedQuestions();

  const { props } = useModalState();

  return (
    <div
      className={cn(
        props.inline &&
          "inline-suggestions-wrapper tv-flex tv-flex-wrap tv-items-center",
      )}
    >
      <p className="component-header tv-m-0 tv-uppercase tv-text-xs tv-pb-2 tv-flex tv-items-center tv-gap-1">
        <button
          onClick={refetchSuggestedQuestion}
          disabled={isLoadingSuggestedQueries}
          className="suggested-question tv-cursor-pointer tv-border tv-rounded-md tv-p-1 tv-text-xs disabled:tv-cursor-not-allowed"
          title="Refresh suggested questions"
        >
          <i className="fa-solid fa-arrow-rotate-right"></i>
        </button>{" "}
        Example questions
      </p>
      <div className={`questions ${props.inline ? "inline-questions" : ""}`}>
        {!props.inline && !suggestedQuestions.length ? (
          <p className="suggested-question empty-state-loading">
            Loading example questions...
          </p>
        ) : null}
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
  );
};
