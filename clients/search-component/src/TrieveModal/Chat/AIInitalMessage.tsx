import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { useChatState } from "../../utils/hooks/chat-context";
import { GroupChatImgCarousel } from "./GroupChatImgCarousel";

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
          <i className="fa-solid fa-wand-magic-sparkles"></i>
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
            {(currentGroup?.name || props.brandName || "Trieve").replace(
              /<[^>]*>/g,
              ""
            )}
          </span>
        </p>
        <GroupChatImgCarousel />
      </span>
      {!messages.length && !currentGroup ? <SuggestedQuestions /> : null}
    </>
  );
};
