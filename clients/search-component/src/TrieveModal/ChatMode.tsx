import React, { useEffect, useRef, useState } from "react";
import Markdown from "react-markdown";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { BackIcon, CheckIcon, LoadingAIIcon, LoadingIcon } from "./icons";
import { Chunk } from "../utils/types";
import { useModalState } from "../utils/hooks/modal-context";

export const ChatMode = () => {
  const { query, setMode, props, modalRef } = useModalState();
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
      const topic = await props.trieve.createTopic({
        name: currentQuestion,
        owner_id: fingerprint.toString(),
      });
      setCurrentQuestion("");
      setCurrentTopic(topic.id);
      createQuestion(topic.id);
    }
  };

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>
  ) => {
    setIsLoading(true);
    let done = false;
    let textInStream = "";

    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        setIsLoading(false);
      } else if (value) {
        const decoder = new TextDecoder();
        const newText = decoder.decode(value);
        textInStream += newText;
        const [text, jsonData] = textInStream.split("||");
        let json;
        try {
          json = JSON.parse(jsonData);
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
        } catch (_) {
          json = null;
        }

        setMessages((m) => [
          [
            m[0][0],
            {
              type: "system",
              text: text,
              additional: json ? json : null,
            },
          ],
          ...m.slice(1),
        ]);
      }
    }
  };

  const createQuestion = async (id?: string) => {
    setIsLoading(true);
    const reader = await props.trieve.createMessageReader({
      topic_id: id || currentTopic,
      new_message_content: currentQuestion,
      llm_options: {
        completion_first: true,
      },
    });
    handleReader(reader);
  };

  useEffect(() => {
    createTopic();
  }, []);

  return (
    <>
      <div className="input-wrapper chat">
        <button onClick={() => setMode("search")} className="back-icon">
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
            modalRef.current?.scroll({ top: 0, behavior: "smooth" });
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
          <ul className="chat-ul" key={i}>
            {chat.map((message, idx) => {
              return (
                <div key={idx}>
                  {message.type == "user" ? (
                    <div className={message.type}>
                      {isLoading && i === 0 ? <LoadingAIIcon /> : <CheckIcon />}
                      <span> {message.text}</span>
                    </div>
                  ) : null}
                  {isLoading && i === 0 && !message.text ? (
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
                      {message.additional && i % 2 == 0 ? (
                        <div className="additional-links">
                          {message.additional
                            .filter(
                              (m) =>
                                (m.metadata.title || m.metadata.page_title) &&
                                m.link
                            )
                            .map((link) => (
                              <a href={link.link as string}>
                                {link.metadata.title ||
                                  link.metadata.page_title}
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
