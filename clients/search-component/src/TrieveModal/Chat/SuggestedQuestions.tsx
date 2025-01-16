import React from "react";
import { useSuggestedQuestions } from "../../utils/hooks/useSuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import { cn } from "../../utils/styles";
import { useAutoAnimate } from "@formkit/auto-animate/react";
import { SparklesIcon } from "../icons";

export const SuggestedQuestions = () => {
  const { askQuestion, setCurrentQuestion } = useChatState();
  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
    refetchSuggestedQuestion,
  } = useSuggestedQuestions();

  const { props } = useModalState();
  const [parent] = useAutoAnimate();

  return (
    <div
      className={cn(
        props.inline &&
          "tv-flex tv-gap-x-3 tv-flex-wrap tv-items-center",
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
          onClick={refetchSuggestedQuestion}
          disabled={isLoadingSuggestedQueries}
          className="suggested-question tv-cursor-pointer tv-border tv-rounded-md tv-p-1 tv-text-xs disabled:tv-cursor-not-allowed tv-text-center"
          title="Refresh suggested questions"
        >
          <i className="fa-solid fa-arrow-rotate-right"></i>
        </button>{" "}
        {!props.inline && !suggestedQuestions.length && (
          <p className="suggested-question tv-text-nowrap empty-state-loading">
            Loading example questions...
          </p>
        )}
        {suggestedQuestions?.map((q) => (
          <button
            onClick={() => {
              setCurrentQuestion(q);
              askQuestion(q);
            }}
            key={q}
            className={`suggested-question tv-flex tv-gap-1 tv-items-center ${
              isLoadingSuggestedQueries ? "loading" : ""
            }`}
          >
            <SparklesIcon width={15} height={15} />
            {q}
          </button>
        ))}
      </div>
    </div>
  );
};
