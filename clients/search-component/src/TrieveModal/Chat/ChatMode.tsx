import React, { Suspense, useEffect, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { AIInitialMessage } from "./AIInitalMessage";
import { useChatState } from "../../utils/hooks/chat-context";
import { ChatMessage } from "./ChatMessage";
import { Tags } from "../Tags";

export const ChatMode = () => {
  const {
    props,
    setMode,
    modalRef,
    open,
    setOpen,
    mode,
    currentGroup,
    setCurrentGroup,
  } = useModalState();
  const {
    askQuestion,
    messages,
    currentQuestion,
    cancelGroupChat,
    setCurrentQuestion,
    clearConversation,
    isDoneReading,
    stopGeneratingMessage,
  } = useChatState();

  const chatInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (mode == "chat" && open) {
      chatInput.current?.focus();
    }
  }, [chatInput, mode, open]);

  return (
    <Suspense>
      <div className={`chat-outer-wrapper ${props.inline ? "": "chat-outer-popup"}`} ref={modalRef}>

        {!props.inline &&
        <div
          className={`close-modal-button chat ${props.type}`}
          onClick={() => setOpen(false)}
        >
          <svg
            className="close-icon"
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
          <span>Close</span>
        </div>}
        <div
          className={`system-information-wrapper${
            currentGroup ? " with-group" : ""
          }`}
        >
          <div className="ai-message">
            <div className="chat-modal-wrapper">
              <div className="ai-message initial-message">
                <AIInitialMessage />
              </div>
              {messages.map((chat, i) => (
                <div key={i} className="message-wrapper">
                  {chat.map((message, idx) => (
                    <ChatMessage key={idx} idx={idx} message={message} />
                  ))}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
      <div
        className={`chat-footer-wrapper${
          messages.length ? " with-messages" : ""
        }`}
      >
        {currentGroup && (
          <div className="chat-group-disclaimer">
            <div>Chatting with {currentGroup.name.replace(/<[^>]*>/g, "")}</div>
            <button
              onClick={() => {
                cancelGroupChat();
              }}
            >
              <i className="fa-solid fa-xmark"></i>
            </button>
          </div>
        )}
        <div className="input-wrapper chat">
          <button
            onClick={() => {
              if (currentGroup) {
                setCurrentGroup(null);
              }
              setMode("search");
            }}
            className="back-icon"
          >
            <i className="fa-solid fa-chevron-left"></i>
          </button>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (currentQuestion) {
                askQuestion(currentQuestion);
              }
            }}
          >
            <input
              ref={chatInput}
              value={currentQuestion}
              onChange={(e) => setCurrentQuestion(e.target.value)}
              placeholder={`Ask me anything about${
                props.brandName ? ` ${props.brandName}` : ""
              }`}
            />
          </form>
        </div>
        <div className={`trieve-footer chat ${props.type}`}>
          {currentQuestion || messages.length ? (
            <div className="chat-controls-row">
              <button
                onClick={() =>
                  currentQuestion
                    ? askQuestion(currentQuestion)
                    : isDoneReading
                      ? clearConversation()
                      : stopGeneratingMessage()
                }
                className="clear-button"
              >
                {currentQuestion ? "Enter" : isDoneReading ? "Clear" : "Stop"}
              </button>
            </div>
          ) : null}
          <div className="tags-row">
            {props.tags?.length ? <Tags /> : null}
            <div className="tags-spacer"></div>
            <a
              className="trieve-powered text-right"
              href="https://trieve.ai"
              target="_blank"
              rel="noopener noreferrer"
            >
              <img
                src="https://cdn.trieve.ai/trieve-logo.png"
                alt="logo"
                className="inline-block mr-2"
              />
              Powered by Trieve
            </a>
          </div>
        </div>
      </div>
    </Suspense>
  );
};

export default ChatMode;
