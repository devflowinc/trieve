// In your ChatMode component
import React, { Suspense, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { useChatState } from "../../utils/hooks/chat-context";
import { ResponseMessage } from "./ResponseMessage";
import { useIntersectionObserver } from "react-intersection-observer-hook";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { FollowupQueries } from "./FollowupQueries";
import { UserMessage } from "./UserMessage";
import { InlineChatHeader } from "./InlineChatHeader";
import { ChatInput } from "./ChatInput";

export const ChatMode = () => {
  const { props, modalRef, minHeight, resetHeight, addHeight } = useModalState();
  const { messages } = useChatState();

  const actualChatRef = useRef<HTMLDivElement>(null);
  const rulerRef = useRef<HTMLDivElement>(null);

  const [ref, { entry }] = useIntersectionObserver();
  const isOnScreen = entry && entry.isIntersecting;

  const onMessageSend = () => {
    setTimeout(() => {
      if (!actualChatRef.current || !modalRef.current) {
        return;
      }

      const userMessages = actualChatRef.current.querySelectorAll(
        ".user-message-container",
      );
      if (userMessages.length === 0) {
        return;
      }

      const lastUserMessage = userMessages[userMessages.length - 1];

      const messageRect = lastUserMessage.getBoundingClientRect();
      const containerRect = modalRef.current.getBoundingClientRect();

      const bufferSpace = 20;

      const scrollTo =
        messageRect.top -
        containerRect.top +
        modalRef.current.scrollTop -
        bufferSpace;

      handleHeightAddition();

      setTimeout(() => {
        if (!modalRef.current) {
          return;
        }
        modalRef.current.scrollTo({
          top: scrollTo,
          behavior: "smooth",
        });
      }, 30); // 30 used for consistency with react-dom updates
    }, 100);
  };

  const calculateHeightToAdd = () => {
    if (!modalRef.current || !actualChatRef.current) {
      return 0;
    }
    if (!rulerRef.current) {
      return 0;
    }

    const userMessages = actualChatRef.current.querySelectorAll(
      ".user-message-container",
    );
    if (userMessages.length === 0) {
      return 0;
    }

    const lastUserMessage = userMessages[userMessages.length - 1];

    const messageRect = lastUserMessage.getBoundingClientRect();

    const scrollContainerVisibleHeight = modalRef.current.clientHeight;
    const messageRectYDistance =
      messageRect.top +
      modalRef.current.scrollTop +
      lastUserMessage.scrollHeight;

    const redLead = rulerRef.current.scrollHeight - messageRectYDistance;

    const targetGap =
      scrollContainerVisibleHeight - lastUserMessage.scrollHeight;

    const heightToAdd = targetGap - redLead;
    return heightToAdd - 80;
  };

  const handleHeightAddition = () => {
    const height = calculateHeightToAdd();
    addHeight(height);
  };

  return (
    <Suspense>
      {props.previewTopicId == undefined && 
      <InlineChatHeader resetHeight={resetHeight} />}
      <div
        ref={modalRef}
        className="chat-modal-wrapper tv-justify-items-stretch tv-flex-grow tv-pt-3 tv-pb-2 tv-px-2 tv-relative tv-overflow-y-auto tv-flex tv-overflow-x-hidden"
      >
        <ChatRuler rulerRef={rulerRef} minHeight={minHeight} />
        <div
          className="tv-flex-col tv-h-full tv-grow tv-flex tv-gap-4 tv-max-w-full"
          ref={actualChatRef}
        >
          {/* Only shows with zero messages */}
          <SuggestedQuestions onMessageSend={onMessageSend} />{" "}
          {messages.map((message, i) => {
            if (message.type === "user") {
              return <UserMessage key={i} message={message} idx={i} />;
            } else {
              return <ResponseMessage key={i} message={message} idx={i} />;
            }
          })}
          <FollowupQueries />
          <div
            ref={ref}
            className="tv-z-50 tv-mx-4 tv-w-4 tv-min-h-1 tv-h-1"
          ></div>
        </div>
      </div>
      <ChatInput onMessageSend={onMessageSend} showShadow={!isOnScreen} />
    </Suspense>
  );
};

// sits on the left side of chat in a flexbox to enforce the minimum height and control scroll
const ChatRuler = ({
  minHeight,
  rulerRef,
}: {
  minHeight: number;
  rulerRef: React.RefObject<HTMLDivElement>;
}) => {
  return (
    <div
      ref={rulerRef}
      className="tv-min-w-[1px]"
      style={{
        minHeight,
      }}
    ></div>
  );
};

export default ChatMode;
