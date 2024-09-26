import React from "react";
import { BackIcon, ReloadIcon } from "./icons";
import { useModalState } from "../utils/hooks/modal-context";
import { useSuggestedQuestions } from "../utils/hooks/useSuggestedQuestions";
import { useChat } from "../utils/hooks/useChat";
import { ChatMessage } from "./ChatMessage";

export const ChatMode = () => {
  const { setMode } = useModalState();
  const { askQuestion, messages, currentQuestion, setCurrentQuestion } =
    useChat();
  const {
    suggestedQuestions,
    isLoadingSuggestedQueries,
    refetchSuggestedQuestion,
  } = useSuggestedQuestions();

  return (
    <div className="chat-outer-wrapper">
      {messages.length ? (
        <div className="chat-modal-wrapper">
          {messages.map((chat, i) => (
            <ul className="chat-ul" key={i}>
              {chat.map((message, idx) => {
                return (
                  <ChatMessage key={idx} message={message} idx={idx} i={i} />
                );
              })}
            </ul>
          ))}
        </div>
      ) : (
        <div className="suggested-questions-wrapper">
          <p>
            Hi! I'm an AI assistant trained on documentation. Ask me anything
          </p>
          <h6>Here are some example questions to get you started</h6>
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
                      isLoadingSuggestedQueries ? " loading" : ""
                    }`}
                  >
                    {q}
                  </button>
                ))}
                <button
                  onClick={refetchSuggestedQuestion}
                  disabled={isLoadingSuggestedQueries}
                  className="suggested-question refetch"
                  title="Refresh suggested questions"
                >
                  <ReloadIcon width="14" height="14" /> Fetch more questions
                </button>
              </>
            ) : null}
          </div>
        </div>
      )}
      <div className="input-wrapper chat">
        <button onClick={() => setMode("search")} className="back-icon">
          <BackIcon />
        </button>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            askQuestion(currentQuestion);
          }}
        >
          <input
            value={currentQuestion}
            onChange={(e) => setCurrentQuestion(e.target.value)}
            placeholder="Ask me anything"
          />
        </form>
      </div>
    </div>
  );
};
