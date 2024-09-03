import React, { useEffect, useRef, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import Markdown from "react-markdown";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { BackIcon, CheckIcon, LoadingAIIcon, LoadingIcon } from "./icons";
import { Chunk } from "../utils/types";

export const ChatMode = ({
  query,
  setMode,
  trieve,
  onNewMessage,
}: {
  query: string;
  setMode: (value: string) => void;
  trieve: TrieveSDK;
  onNewMessage: () => void;
}) => {
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const called = useRef(false);
  const [messages, setMessages] = useState<
    {
      type: string;
      text: string;
      additional: Chunk[] | null;
    }[][]
  >([
    [
      {
        type: "user",
        text: query,
        additional: null,
      },
    ],
  ]);
  const [isLoading, setIsLoading] = useState(false);

  const createTopic = async () => {
    if (!currentTopic && !called.current) {
      called.current = true;
      setIsLoading(true);
      const fingerprint = await getFingerprint();
      const topic = await trieve.createTopic({
        first_user_message: currentQuestion,
        owner_id: fingerprint.toString(),
      });
      setCurrentQuestion("");
      setCurrentTopic(topic.id);
      createQuestion(topic.id);
    }
  };

  const createQuestion = async (id?: string) => {
    setIsLoading(true);
    const answer = await trieve.createMessage({
      topic_id: id || currentTopic,
      new_message_content: currentQuestion,
    });
    const [json, text] = answer.split("||");
    setMessages((m) => [
      [
        m[0][0],
        {
          type: "system",
          text: text,
          additional: JSON.parse(json),
        },
      ],
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
          <BackIcon />
        </button>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setMessages((m) => [
              [{ type: "user", text: currentQuestion, additional: null }],
              ...m,
            ]);
            createQuestion();
            setCurrentQuestion("");
            onNewMessage();
          }}
        >
          <input
            value={currentQuestion}
            onChange={(e) => setCurrentQuestion(e.target.value)}
            placeholder="Ask a follow up question"
          />
        </form>
      </div>
      <div className="chat-modal-wrapper">
        {messages.map((chat, i) => (
          <ul className="chat-ul">
            {chat.map((message, idx) => {
              return (
                <div>
                  {message.type == "user" ? (
                    <div className={message.type}>
                      {isLoading && i === 0 ? <LoadingAIIcon /> : <CheckIcon />}
                      <span> {message.text}</span>
                    </div>
                  ) : null}
                  {isLoading && i === 0 ? (
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
                                language={
                                  className?.split("language")[1] || "sh"
                                }
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
                      {message.additional && i === 0 ? (
                        <div className="additional-links">
                          {message.additional
                            .filter(
                              (m) =>
                                m.metadata.title ||
                                (m.metadata.page_title && m.link)
                            )
                            .map((link) => (
                              <a href={link.link as string}>
                                {link.metadata.page_title}
                              </a>
                            ))}
                        </div>
                      ) : null}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </ul>
        ))}
      </div>
    </>
  );
};
