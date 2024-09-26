import * as React from "react";
import { useChat } from "../utils/hooks/useChat";
import { CheckIcon, LoadingAIIcon, LoadingIcon } from "./icons";
import Markdown from "react-markdown";
import SyntaxHighlighter from "react-syntax-highlighter";
import { nightOwl } from "react-syntax-highlighter/dist/esm/styles/hljs";
import { Chunk } from "../utils/types";

type Message = {
  type: string;
  text: string;
  additional: Chunk[] | null;
};

export const ChatMessage = ({
  message,
  idx,
  i,
}: {
  i: number;
  idx: number;
  message: Message;
}) => {
  const { isLoading } = useChat();
  return (
    <div>
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
                  return <code className="single-line">{children}</code>;
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
          {message.additional && i % 2 == 0 ? (
            <div className="additional-links">
              {message.additional
                .filter(
                  (m) => (m.metadata.title || m.metadata.page_title) && m.link
                )
                .map((link) => (
                  <a href={link.link as string}>
                    {link.metadata.title || link.metadata.page_title}
                  </a>
                ))}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
};
