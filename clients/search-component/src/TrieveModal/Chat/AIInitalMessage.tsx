import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { AIIcon } from "../icons";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";

export const AIInitialMessage = () => {
  const { props } = useModalState();
  const { messages } = useChatState();

  return (
    <>
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
      <span className="content">
        <p>Hi!</p>
        <p>
          I'm an AI assistant with access to documentation, help articles, and
          other content.
        </p>
        <p>
          Ask me anything about{" "}
          <span
            style={{
              backgroundColor: props.brandColor ?? "#CB53EB",
            }}
            className="brand-name"
          >
            {props.brandName}
          </span>
        </p>
      </span>
      {!messages.length ? <SuggestedQuestions /> : null}
    </>
  );
};
