import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { GroupChatImgCarousel } from "./GroupChatImgCarousel";
import { SparklesIcon } from "../icons";

export const AIInitialMessage = () => {
  const { props, currentGroup } = useModalState();
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
          <SparklesIcon />
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
          I'm an AI assistant here to help. What can I assist you with today?
        </p>
        <p className="brand-paragraph">
          Ask me anything about{" "}
          <span
            style={{
              backgroundColor: props.brandColor ?? "#CB53EB",
            }}
            className="brand-name"
          >
            {(currentGroup?.name || props.brandName || "Trieve")
              .replace(/<[^>]*>/g, "")
              .split(" ")
              .slice(0, 3)
              .join(" ")}
          </span>{" "}
          {(currentGroup?.name || props.brandName || "Trieve")
            .replace(/<[^>]*>/g, "")
            .split(" ")
            .slice(3)
            .join(" ")}
        </p>
        <GroupChatImgCarousel />
      </span>
      <p> </p>
      {!messages.length && !currentGroup ? <SuggestedQuestions /> : null}
    </>
  );
};
