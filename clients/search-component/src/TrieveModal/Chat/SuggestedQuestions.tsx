import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
export const SuggestedQuestions = () => {
  const { askQuestion, setCurrentQuestion } = useChatState();
  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
    refetchSuggestedQuestion,
  } = useSuggestedQuestions();

  const {
    props
  } = useModalState()

  return (
    <>
      <div className={props.inline ? "inline-suggestions-wrapper": ""}>
        {!props.inline &&
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
        </p>}
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
    </>
  );
};
