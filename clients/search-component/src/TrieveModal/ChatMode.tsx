import React, { useEffect, useRef, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import Markdown from "react-markdown";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { LoadingIcon } from "./icons";

export const ChatMode = ({
  query,
  setMode,
  trieve,
}: {
  query: string;
  setMode: (value: string) => void;
  trieve: TrieveSDK;
}) => {
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const highlighter = useRef<any>(null);
  const [messages, setMessages] = useState([
    {
      type: "user",
      text: query,
    },
  ]);
  const [isLoading, setIsLoading] = useState(false);

  const createTopic = async () => {
    const fingerprint = await getFingerprint();
    const topic = await trieve.createTopic({
      first_user_message: currentQuestion,
      owner_id: fingerprint.toString(),
    });
    setCurrentTopic(topic.id);
    createQuestion(topic.id);
  };

  const createQuestion = async (id?: string) => {
    setIsLoading(true);
    const answer = await trieve.createMessage({
      topic_id: id || currentTopic,
      new_message_content: currentQuestion,
    });
    const [json, text] = answer.split("||");
    setMessages((m) => [
      m[0],
      {
        type: "system",
        text: text,
        additional: JSON.parse(json),
      },
      ...m.slice(1),
    ]);
    setIsLoading(false);
  };

  useEffect(() => {
    createTopic();
  }, []);

  return (
    <>
      <div className="input-wrapper chat">
        <button onClick={() => setMode("search")}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            className="search-icon"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M15 6l-6 6l6 6" />
          </svg>
        </button>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setMessages((m) => [{ type: "user", text: currentQuestion }, ...m]);
            createQuestion();
          }}
        >
          <input
            value={currentQuestion}
            onChange={(e) => setCurrentQuestion(e.target.value)}
            placeholder={"Ask a follow up question"}
          />
        </form>
      </div>
      <ul className="chat-modal-wrapper">
        {messages.map((message, idx) => {
          return (
            <div>
              {message.type == "user" ? (
                <div className={message.type}>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                    <path d="M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-1.293 5.953a1 1 0 0 0 -1.32 -.083l-.094 .083l-3.293 3.292l-1.293 -1.292l-.094 -.083a1 1 0 0 0 -1.403 1.403l.083 .094l2 2l.094 .083a1 1 0 0 0 1.226 0l.094 -.083l4 -4l.083 -.094a1 1 0 0 0 -.083 -1.32z" />
                  </svg>
                  <span> {message.text}</span>
                </div>
              ) : null}
              {isLoading ? (
                <div className="system">
                  <LoadingIcon />
                </div>
              ) : null}
              {message.type === "system" ? (
                <div className="system">
                  <Markdown
                    components={{
                      code: (props) => {
                        const { className, children } = props || {};
                        if (!children) return null;
                        if (!className) {
                          return (
                            <code className="single-line">{children}</code>
                          );
                        }
                        return (
                          <SyntaxHighlighter
                            language={className?.split("language")[1] || "sh"}
                            style={nightOwl}
                          >
                            {children?.toString()}
                          </SyntaxHighlighter>
                        );
                      },
                    }}
                    key={idx}
                  >
                    {message.text}
                  </Markdown>
                </div>
              ) : null}
            </div>
          );
        })}
      </ul>
    </>
  );
};
