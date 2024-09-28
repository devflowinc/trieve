import React from "react";
import { BackIcon, ReloadIcon, AIIcon } from "./icons";
import { useModalState } from "../utils/hooks/modal-context";
import { useSuggestedQuestions } from "../utils/hooks/useSuggestedQuestions";
import { useChat } from "../utils/hooks/useChat";
import { ChatMessage } from "./ChatMessage";

export const ChatMode = () => {
  const { props, setMode } = useModalState();
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
        <div className="system-information-wrapper">
          <div className="ai-message">
            <span className="ai-avatar">
              {props.brandLogoImgSrcUrl ? (
                <img
                  src={props.brandLogoImgSrcUrl}
                  alt={props.brandName || "Brand logo"}
                />
              ) : (
                <AIIcon />
              )}
              <p
                className="tag"
                // style mostly transparent brand color
                style={{
                  backgroundColor: props.brandColor
                    ? `${props.brandColor}18`
                    : "#CB53EB18",
                  color: props.brandColor ?? "#CB53EB",
                }}
              >
                AI assistant
              </p>
            </span>
            <p className="content">
              <p>Hi!</p>
              <p>
                I'm an AI assistant with access to documentation, help articles,
                and other content.
              </p>
              <p>
                Ask me anything about{" "}
                <span
                  style={{ backgroundColor: props.brandColor ?? "#CB53EB" }}
                  className="brand-name"
                >
                  {props.brandName}
                </span>
              </p>
            </p>
          </div>
          <div className="ai-message">
            <p></p>
            <div>
              <p className="header">
                <button
                  onClick={refetchSuggestedQuestion}
                  disabled={isLoadingSuggestedQueries}
                  className="suggested-question refetch"
                  title="Refresh suggested questions"
                >
                  <ReloadIcon width="14" height="14" />
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
                          isLoadingSuggestedQueries ? " loading" : ""
                        }`}
                      >
                        {q}
                      </button>
                    ))}
                  </>
                ) : null}
              </div>
            </div>
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
