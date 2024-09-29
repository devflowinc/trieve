import * as React from "react";
import { LoadingIcon } from "./icons";
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
}: {
  idx: number;
  message: Message;
}) => {
  return (
    <div>
      {message.text == "Loading..." ? (
        <div className="system">
          <LoadingIcon className="loading" />
        </div>
      ) : null}
      {message.type === "system" && message.text != "Loading..." ? (
        <div className="system">
          <Markdown
            components={{
              code: (props) => {
                const { className, children } = props || {};
                if (!children) return null;
                if (!className) {
                  return (<SyntaxHighlighter
                    language={"bash"}
                    style={nightOwl}
                  >
                    {children?.toString()}
                  </SyntaxHighlighter>);
                }
                return (
                  <SyntaxHighlighter
                    language={className?.split("language")[1] || "bash"}
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
          {message.additional ? (
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
