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
import { useChatHeight } from "../../utils/hooks/useChatHeight";

export const ChatMode = () => {
  const { modalRef } = useModalState();
  const { messages } = useChatState();

  const actualChatRef = useRef<HTMLDivElement>(null);
  const rulerRef = useRef<HTMLDivElement>(null);
  const { minHeight, resetHeight, addHeight } = useChatHeight(actualChatRef); // Get contentHeight from the hook

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

      console.log("scrollTo", scrollTo);

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

    console.log("messageRectYDistance", messageRectYDistance);

    console.log("ruler height", rulerRef.current.scrollHeight);
    const redLead = rulerRef.current.scrollHeight - messageRectYDistance;
    console.log("redLead", redLead);

    const targetGap =
      scrollContainerVisibleHeight - lastUserMessage.scrollHeight;

    const heightToAdd = targetGap - redLead;
    console.log(
      `targetGap: ${targetGap}, redLead: ${redLead}, heightToAdd: ${heightToAdd}`,
    );

    // subtract the height of the message itself
    // 40 is the magic number somehow
    console.log("lastUserMessage.scrollHeight", lastUserMessage.scrollHeight);
    return heightToAdd - 80;
  };

  const handleHeightAddition = () => {
    const height = calculateHeightToAdd();
    addHeight(height);
  };

  return (
    <Suspense>
      <InlineChatHeader resetHeight={resetHeight} />
      <div
        ref={modalRef}
        className="chat-modal-wrapper tv-justify-items-stretch tv-flex-grow tv-pt-3 tv-pb-2 tv-px-2 tv-relative tv-overflow-y-auto tv-flex"
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
