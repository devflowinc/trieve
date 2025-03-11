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
  const { minHeight, resetHeight, addHeight } = useChatHeight(actualChatRef);

  const [ref, { entry }] = useIntersectionObserver();
  const isOnScreen = entry && entry.isIntersecting;

  const onMessageSend = () => {
    // Ensure we have enough height for new messages
    addHeight(800);

    // We need to scroll after the DOM updates with the new message
    // Using setTimeout to ensure this happens after React's render cycle
    setTimeout(() => {
      if (!actualChatRef.current || !modalRef.current) {
        return;
      }

      // Find the user message that was just added (the last user message)
      const userMessages = actualChatRef.current.querySelectorAll(
        ".user-message-container",
      );
      if (userMessages.length === 0) {
        return;
      }

      const lastUserMessage = userMessages[userMessages.length - 1];

      // Calculate position to scroll to - we want the user message at the top of the viewport
      const messageRect = lastUserMessage.getBoundingClientRect();
      const containerRect = modalRef.current.getBoundingClientRect();

      // Calculate the scroll position - message position relative to the scrollable container
      const scrollTo =
        messageRect.top - containerRect.top + modalRef.current.scrollTop;

      // Scroll the modal container
      modalRef.current.scrollTo({
        top: scrollTo,
        behavior: "smooth",
      });
    }, 100);
  };

  return (
    <Suspense>
      <InlineChatHeader resetHeight={resetHeight} />
      <div
        ref={modalRef}
        className="chat-modal-wrapper tv-justify-items-stretch tv-flex-grow tv-pt-3 tv-pb-2 tv-px-2 tv-relative tv-overflow-y-auto tv-flex"
      >
        <ChatRuler minHeight={minHeight} />
        <div
          className="tv-flex-col tv-h-full tv-grow tv-flex tv-gap-4 tv-max-w-full"
          ref={actualChatRef}
        >
          <SuggestedQuestions /> {/* Only shows with zero messages */}
          {messages.map((message, i) => {
            if (message.type === "user") {
              return <UserMessage key={i} message={message} idx={i} />;
            } else {
              return <ResponseMessage key={i} message={message} idx={i} />;
            }
          })}
          <FollowupQueries />
          <button onClick={onMessageSend}>Add height - {minHeight}</button>
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
const ChatRuler = ({ minHeight }: { minHeight: number }) => {
  return (
    <div
      className="tv-bg-red-500 tv-min-w-[8px]"
      style={{
        minHeight,
      }}
    ></div>
  );
};

export default ChatMode;
