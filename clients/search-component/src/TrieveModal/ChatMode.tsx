import React from "react";
import { BackIcon, ReloadIcon, AIIcon, UserIcon } from "./icons";
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
      <div className="system-information-wrapper">
        <div className="ai-message">
          <>
            <div className="chat-modal-wrapper">
              <div className="scrollable-content">
                <div className="ai-message initial-message">
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
                      }}>
                      AI assistant
                    </p>
                  </span>
                  <p className="content">
                    <p>Hi!</p>
                    <p>
                      I'm an AI assistant with access to documentation, help
                      articles, and other content.
                    </p>
                    <p>
                      Ask me anything about{" "}
                      <span
                        style={{
                          backgroundColor: props.brandColor ?? "#CB53EB",
                        }}
                        className="brand-name">
                        {props.brandName}
                      </span>
                    </p>
                  </p>
                  {!messages.length ? (
                    <>
                      <p></p>
                      <div>
                        <p className="header">
                          <button
                            onClick={refetchSuggestedQuestion}
                            disabled={isLoadingSuggestedQueries}
                            className="suggested-question refetch"
                            title="Refresh suggested questions">
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
                                    isLoadingSuggestedQueries ? "loading" : ""
                                  }`}>
                                  {q}
                                </button>
                              ))}
                            </>
                          ) : null}
                        </div>
                      </div>
                    </>
                  ) : null}
                  {messages.map((chat) => (
                    <div className="message-wrapper">
                      {chat.map((message, idx) => {
                        return (
                          <>
                            {message.type == "user" ? (
                              <>
                                <span className="ai-avatar user">
                                  <UserIcon />
                                  <p
                                    className="tag"
                                    // style mostly transparent brand color
                                    style={{
                                      backgroundColor: props.brandColor
                                        ? `${props.brandColor}18`
                                        : "#CB53EB18",
                                      color: props.brandColor ?? "#CB53EB",
                                    }}>
                                    User
                                  </p>
                                </span>
                                <div className={message.type}>
                                  <span> {message.text}</span>
                                </div>
                              </>
                            ) : (
                              <>
                                <span className="ai-avatar assistant">
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
                                    }}>
                                    AI assistant
                                  </p>
                                </span>
                                <ChatMessage
                                  key={idx}
                                  message={message}
                                  idx={idx}
                                />
                              </>
                            )}
                          </>
                        );
                      })}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </>
        </div>
      </div>

      <div className="input-wrapper chat">
        <button onClick={() => setMode("search")} className="back-icon">
          <BackIcon />
        </button>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            askQuestion(currentQuestion);
          }}>
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
