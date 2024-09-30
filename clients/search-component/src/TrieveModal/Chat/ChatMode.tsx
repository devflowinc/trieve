import React from "react";
import { BackIcon } from "../icons";
import { useModalState } from "../../utils/hooks/modal-context";
import { AIInitialMessage } from "./AIInitalMessage";
import { useChatState } from "../../utils/hooks/chat-context";
import { ChatMessage } from "./ChatMessage";

export const ChatMode = () => {
  const { setMode } = useModalState();
  const { askQuestion, messages, currentQuestion, setCurrentQuestion } =
    useChatState();

  return (
    <>
      <div className="chat-outer-wrapper">
        <div className="system-information-wrapper">
          <div className="ai-message">
            <div className="chat-modal-wrapper">
              <div className="scrollable-content">
                <div className="ai-message initial-message">
                  <AIInitialMessage />
                  {messages.map((chat) => (
                    <div className="message-wrapper">
                      {chat.map((message, idx) => (
                        <ChatMessage key={idx} idx={idx} message={message} />
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="chat-footer-wrapper">
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
        <div className={`footer chat`}>
          <div className="bottom-row">
            <span className="spacer" />
            <a
              className="trieve-powered"
              href="https://trieve.ai"
              target="_blank"
            >
              <img src="https://cdn.trieve.ai/trieve-logo.png" alt="logo" />
              Powered by Trieve
            </a>
          </div>
        </div>
      </div>
    </>
  );
};
